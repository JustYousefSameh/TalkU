package com.talku.Infrastruture.Wireguard;

import java.util.Base64;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.WString;
import com.talku.Utils.PathHelpers;

public class WireguardTunnel {
    public interface TunnelLibrary extends Library {
        TunnelLibrary INSTANCE = Native.load("tunnel", TunnelLibrary.class);

        boolean WireGuardTunnelService(WString configPath);

        boolean WireGuardGenerateKeypair(byte[] publicKey, byte[] privateKey);

    }

    public static void startWireguard(String configPath) {
        WString configPathWString = new WString(configPath);

        System.load(PathHelpers.getWireguardDllPath());
        TunnelLibrary.INSTANCE.WireGuardTunnelService(configPathWString);
    }

    public static class Keypair {
        private final String publicKey;
        private final String privateKey;

        public Keypair(String publicKey, String privateKey) {
            this.publicKey = publicKey;
            this.privateKey = privateKey;
        }

        public String getPublicKey() {
            return publicKey;
        }

        public String getPrivateKey() {
            return privateKey;
        }
    }

    // Generate WireGuard key pair
    public static Keypair generate() {
        try {
            System.out.println("Generating WireGuard key pair...");
            byte[] publicKey = new byte[32];
            byte[] privateKey = new byte[32];

            boolean success = TunnelLibrary.INSTANCE.WireGuardGenerateKeypair(publicKey, privateKey);

            return new Keypair(Base64.getEncoder().encodeToString(publicKey),
                    Base64.getEncoder().encodeToString(privateKey));

        } catch (Exception e) {
            System.out.println(e);
            throw new RuntimeException("Failed to generate WireGuard key pair.", e);
        } finally {
            System.out.println("Key pair generation completed.");
        }
    }
}
