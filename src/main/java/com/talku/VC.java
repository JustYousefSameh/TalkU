package com.talku;

import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.net.InetAddress;
import java.net.URI;
import java.net.URL;
import java.nio.charset.StandardCharsets;
import java.util.List;

import org.apache.commons.csv.CSVFormat;
import org.apache.commons.csv.CSVParser;
import org.apache.commons.csv.CSVRecord;

import com.profesorfalken.jpowershell.PowerShell;

import io.vavr.control.Either;
import javafx.beans.property.SimpleStringProperty;
import javafx.beans.property.StringProperty;

public class VC {

    public class VCException extends Exception {
        public VCException(String message) {
            super(message);
        }
    }

    private static StringProperty labelProperty = new SimpleStringProperty();

    private String hostName;
    private String ip;
    private PowerShell powerShell;

    public StringProperty labelProperty() {
        return labelProperty;
    }

    public VC() {
        powerShell = PowerShell.openSession();
        System.out.println("VC created");
    }

    public Either<VCException, Boolean> connect(Boolean isConnected) {
        try {
            if (!isConnected) {
                if (doesAdapterExist()) {
                    if (checkingIP()) {
                        connectVPN();
                        maskIp();
                    } else {
                        fetchIp();
                        setIpAdapter();
                        connectVPN();
                        maskIp();
                    }
                } else {
                    fetchIp();
                    makeAdapter();
                    connectVPN();
                    maskIp();
                }
                return Either.right(true);
            } else {
                unMaskAndDisconnect();
                return Either.right(false);
            }
        } catch (VCException e) {
            System.out.println(e.getMessage());
            return Either.left(e);
        }

    }

    private void updateLabel(String text) {
        labelProperty.set(text);
        // javafx.application.Platform.runLater(() -> {
        // VCE.animateLabelChange(text);
        // });
    }

    public Boolean checkingIP() {
        updateLabel("Checking IP...");
        System.out.println("Checking IP...");
        String command = "(Get-VpnConnection -Name \"TalkU\").ServerAddress";
        String output = powerShell.executeCommand(command).getCommandOutput();
        if (output.equals("")) {
            return false;
        }
        try {
            InetAddress meow = InetAddress.getByName(output);
            return meow.isReachable(5000);
        } catch (Exception e) {
            return false;
        }

    }

    public void fetchIp() throws VCException {

        updateLabel("Fetching IP...");

        try {
            final String csvUrl = "http://www.vpngate.net/api/iphone/";

            final URL url = new URI(csvUrl).toURL();
            BufferedReader reader = new BufferedReader(new InputStreamReader(url.openStream(), StandardCharsets.UTF_8));

            CSVParser csvParser = CSVFormat.DEFAULT.withHeader().parse(reader);
            List<CSVRecord> records = csvParser.getRecords();

            for (int i = 1; i < records.size(); i++) {
                CSVRecord csvRecord = records.get(i);
                InetAddress meow = InetAddress.getByName(csvRecord.get(0) + ".opengw.net");

                if (meow.isReachable(5000)) {
                    hostName = csvRecord.get(0) + ".opengw.net";
                    break;
                }
            }
        } catch (Exception e) {
            throw new VCException("Failed to fetch IP");
        }
    }

    public void makeAdapter() {
        updateLabel("Creating Adapter...");

        String command = String.format("Add-VpnConnection -Name TalkU -ServerAddress %s -TunnelType Sstp"
                + " -Force -EncryptionLevel Optional -AuthenticationMethod Pap, Chap, MSChapv2 -RememberCredential -PassThru -SplitTunneling",
                hostName);
        System.out.println(powerShell.executeCommand(command).getCommandOutput());
    }

    public Boolean doesAdapterExist() {

        String command = "Get-VpnConnection -Name TalkU";
        String output = powerShell.executeCommand(command).getCommandOutput();
        // Notice that here having an error probably means that the adapter does not
        // exist
        return !output.contains("not found");
    }

    public void setIpAdapter() throws VCException {

        updateLabel("Setting IP...");
        String command = String.format("Set-VpnConnection -Name TalkU -ServerAddress %s", hostName);
        String.format("Setting %s to adapter", hostName);
        powerShell.executeCommand(command);
        if (powerShell.isLastCommandInError()) {
            throw new VCException("Failed to set IP");
        }
    }

    public void maskIp() throws VCException {

        System.out.println("Masking IP...");
        // updateLabel("Routing IPs...");
        ip = powerShell.executeCommand(
                "(netsh interface ipv4 show addresses \"TalkU\" | findstr \"IP\") -match '\\s+IP Address:\\s+(\\S+)' | Out-Null \r\n"
                        + "$matches[1]")
                .getCommandOutput();

        if (powerShell.isLastCommandInError()) {
            throw new VCException("Failed to get IP");
        }

        String command = String.format("route add 188.0.0.0 mask 255.0.0.0 %1$s" // + //
        // "route add 63.251.140.0 mask 255.255.255.0 %1$s\r\n" + //
        // "route add 69.25.124.0 mask 255.255.255.0 %1$s\r\n" + //
        // "route add 70.42.0.0 mask 255.255.0.0 %1$s\r\n" + //
        // "route add 74.201.0.0 mask 255.255.0.0 %1$s\r\n" + //
        // "route add 188.42.0.0 mask 255.255.0.0 %1$s\r\n" + //
        // "route add 216.52.0.0 mask 255.255.0.0 %1$s\r\n" + //
        // "route add 85.0.0.0 mask 255.0.0.0 %1$s\r\n" + //
        // "route add 35.201.0.0 mask 255.255.0.0 %1$s\r\n" + //
        // "route add 151.0.0.0 mask 255.0.0.0 %1$s\r\n" + //
        // "route add 18.0.0.0 mask 255.0.0.0 %1$s\r\n" + //
        // "route add 37.0.0.0 mask 255.0.0.0 %1$s\r\n" + //
        // "route add 3.0.0.0 mask 255.0.0.0 %1$s\r\n" + //
        // "route add 34.0.0.0 mask 255.0.0.0 %1$s\r\n"
                , ip);
        System.out.println(powerShell.executeCommand(command).getCommandOutput());
        // powerShell.executeCommand(command);
        if (powerShell.isLastCommandInError()) {
            throw new VCException("Failed to Mask IP");
        }
    }

    public void connectVPN() throws VCException {
        updateLabel("Connecting...");
        System.out.println("Connecting...");
        String command = "rasdial TalkU vpn vpn";
        powerShell.executeCommand(command);
        if (powerShell.isLastCommandInError()) {
            throw new VCException("Failed to connect to VPN");
        }

    }

    public void unMaskAndDisconnect() {
        // No need to unMask as the routes are deleted when the adapter is disconnected
        String command = "rasdial TalkU /disconnect";
        powerShell.executeCommand(command);
        // if (powerShell.isLastCommandInError()) {
        // throw new VCException("Failed to Disconnect");
        // }
    }

    public void close() {
        System.out.println("Closing...");
        if (powerShell != null) {
            powerShell.close();
        }
        powerShell = null;
    }

}
