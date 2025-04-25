package com.talku.Controller;

import java.io.File;
import java.io.IOException;
import java.net.InetAddress;
import java.net.URI;
import java.net.URISyntaxException;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.net.http.HttpResponse.BodyHandlers;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.List;

import com.talku.Infrastruture.VpnConfig.ConfigHandler;
import com.talku.Infrastruture.Wireguard.WireGuardAdapter;
import com.talku.Infrastruture.Wireguard.WireGuardTunnelService;
import com.talku.Infrastruture.Wireguard.WireGuardAdapter.WireGuardException;
import com.talku.Utils.PathHelpers;

import io.vavr.control.Either;
import javafx.beans.property.SimpleStringProperty;
import javafx.beans.property.StringProperty;

// First time connecting (config file doesn't exist)
// Create config file by communication with api

// Connecting 
// Add service to windows and run it
// Watch adapter handshake (if a handshake happened then we're connected)

// Disconnecting
// Stop and remove the service from windows

public class TalkUController {

    abstract public static class VCException extends Exception {
        public VCException(String message) {
            super(message);
        }
    }

    private static int retryCount = 0;

    public static Either<VCException, Boolean> connect(Boolean isConnected) {

        // Adding the registry key to allow script execution (Postup and Postdown in
        // wireguard config)
        // This only needs to run one time
        // Checking for log.bin file because if it's not there it means this is the
        // first time this app is running
        if (!Files.exists(Paths.get(PathHelpers.getLogPath()))) {
            ProcessBuilder processBuilder = new ProcessBuilder("reg", "add", "HKLM\\Software\\WireGuard", "/v",
                    "DangerousScriptExecution", "/t", "REG_DWORD", "/d", "1", "/f");
            try {
                Process process = processBuilder.start();
            } catch (IOException e) {
                System.out.println("Error adding registry key: " + e.getMessage());
                return Either.left(new ConfigHandler.ConfigException("Failed to add registry key"));
            }
        }

        if (isConnected) {
            stopAndRemoveService();
            return Either.right(false);
        }

        if (!Files.exists(Paths.get(PathHelpers.getConfigPath()))) {
            Either<ConfigHandler.ConfigException, Void> configOrError = ConfigHandler.getConfigAndWriteToFile();
            if (configOrError.isLeft()) {
                return Either.left(configOrError.getLeft());
            }
        }

        addAndStartService();

        Either<WireGuardException, Void> exceptionOrConnected = WireGuardAdapter.CheckIsConnected();
        if (exceptionOrConnected.isLeft()) {

            if (exceptionOrConnected.getLeft() instanceof WireGuardAdapter.WireguardHandshakeException) {
                retryCount++;
                // if connection failed 3 times, then the config probably is invalid
                // Delete the config file and a new one will be created next time the user tries
                // to connect
                if (retryCount == 3) {
                    try {
                        System.out.println("Deleting config file");
                        Files.deleteIfExists(Paths.get(PathHelpers.getConfigPath()));
                    } catch (IOException e) {
                        return Either.left(new ConfigHandler.ConfigException("Failed to delete config file"));
                    }
                }
            }

            // Stop and remove the service if the connection failed
            stopAndRemoveService();
            return Either.left(exceptionOrConnected.getLeft());
        }

        // Reset retry count after successful connection
        retryCount = 0;
        return Either.right(true);
    }

    private static Either<VCException, Void> stopAndRemoveService() {
        WireGuardTunnelService wireGuardTunnelService = new WireGuardTunnelService();
        wireGuardTunnelService.stop();
        wireGuardTunnelService.uninstall();

        return Either.right(null);
    }

    // TODO error handeling
    private static Either<VCException, Void> addAndStartService() {
        WireGuardTunnelService wireGuardTunnelService = new WireGuardTunnelService();

        String[] deps = { "Nsi", "TcpIp" };

        String jarPath = PathHelpers.getJarPath();
        String javaPath = PathHelpers.getJavaPath();
        String configPath = PathHelpers.getConfigPath();

        // Path and args for the service to run Looks like this:
        // "javaPath" -jar "jarPath" /service "configPath"
        // The "" is needed
        String pathAndArgs = String.format("\"%1$s\" -jar \"%2$s\" /service \"%3$s\"", javaPath, jarPath, configPath);
        wireGuardTunnelService.install("TalkU", "The Wireguard Service for TalkU", deps, null, null, pathAndArgs);
        wireGuardTunnelService.start();
        return Either.right(null);
    }
}
