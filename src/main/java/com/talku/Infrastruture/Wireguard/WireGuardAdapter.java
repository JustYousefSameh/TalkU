package com.talku.Infrastruture.Wireguard;

import java.util.Arrays;
import java.util.Base64;
import java.util.List;
import java.util.Timer;
import java.util.TimerTask;
import java.io.IOException;
import java.nio.file.*;

import com.sun.jna.ptr.IntByReference;
import com.talku.Controller.TalkUController.VCException;
import com.talku.Infrastruture.VpnConfig.ConfigHandler;
import com.talku.Utils.PathHelpers;

import io.vavr.control.Either;

import com.sun.jna.*;
import com.sun.jna.platform.unix.aix.Perfstat.perfstat_process_t;

import java.util.Date;

public class WireGuardAdapter {

    public abstract static class WireGuardException extends VCException {
        public WireGuardException(String message) {
            super(message);
        }
    }

    public static class WireGuardServiceException extends WireGuardException {
        public WireGuardServiceException(String message) {
            super(message);
        }
    }

    public static class WireguardHandshakeException extends WireGuardException {
        public WireguardHandshakeException(String message) {
            super(message);
        }
    }

    public static Either<WireGuardException, Void> CheckIsConnected() {
        Pointer adapter = null;

        try {
            // Same as config name
            WString adapterName = new WString("talkuwg");
            adapter = WireGuardLibrary.INSTANCE.WireGuardOpenAdapter(adapterName);

            // True when fucntion reaches the handshake part
            // Upon it we send different excetions
            // if a handshake exception is thrown three times in a row then we assume that
            // the config is invalid
            boolean handshakeReached = false;

            // Timeout for handshake connection
            long startTime = System.currentTimeMillis();
            long timeout = 10_000;

            while (adapter == null && (System.currentTimeMillis() - startTime < timeout)) {
                System.out.println("Waiting for adapter to be created...");
                adapter = WireGuardLibrary.INSTANCE.WireGuardOpenAdapter(adapterName);
                Thread.sleep(200);
            }

            while (System.currentTimeMillis() - startTime < timeout) {
                handshakeReached = false;

                int guess = 1024;
                IntByReference sizeRef = new IntByReference(guess);
                byte[] buffer = null;

                System.out.println("Waiting for configuration to be available...");
                buffer = new byte[sizeRef.getValue()];
                if (!WireGuardLibrary.INSTANCE.WireGuardGetConfiguration(adapter, buffer, sizeRef)) {
                    int err = Native.getLastError();
                    if (err != 234) {
                        System.err.println("Error: " + err);
                        WireGuardLibrary.INSTANCE.WireGuardCloseAdapter(adapter);
                        return Either.left(new WireGuardServiceException("Failed to connect to VPN."));
                    }
                    // Loop again if no configuration found
                    continue;
                }

                Memory mem = new Memory(buffer.length);
                mem.write(0, buffer, 0, buffer.length);

                WireGuardStructs.IoctlInterface iface = new WireGuardStructs.IoctlInterface(mem);
                long offset = iface.size();

                WireGuardStructs.IoctlPeer peer = new WireGuardStructs.IoctlPeer(mem.share(offset));

                System.out.println("Waiting for handshake...");

                // Handshake means successful connection
                if (peer.LastHandshake != 0) {
                    System.out.println("Handshake successful");
                    WireGuardLibrary.INSTANCE.WireGuardCloseAdapter(adapter);
                    return Either.right(null);
                }

                handshakeReached = true;
                Thread.sleep(500);
            }
            System.out.println("no Handshake after 10 seconds");
            WireGuardLibrary.INSTANCE.WireGuardCloseAdapter(adapter);

            if (handshakeReached) {
                return Either.left(new WireguardHandshakeException("Failed to connect to VPN."));
            } else {
                return Either.left(new WireGuardServiceException("Failed to connect to VPN."));
            }

        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            if (adapter != null) {
                WireGuardLibrary.INSTANCE.WireGuardCloseAdapter(adapter);
            }
            return Either.left(new WireGuardServiceException("Failed to connect to VPN."));
        }

    }
}
