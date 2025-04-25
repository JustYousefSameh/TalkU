package com.talku.Infrastruture.Wireguard;

import java.util.Arrays;
import java.util.Base64;
import java.util.List;

import com.talku.Utils.PathHelpers;
import com.sun.jna.*;

public class WireGuardStructs {

    public static class IoctlInterface extends Structure {
        public static class ByReference extends IoctlInterface implements Structure.ByReference {
            public ByReference(Memory mem) {
                super(mem);
            }
        }

        public int Flags;
        public short ListenPort;
        public byte[] PrivateKey = new byte[32];
        public byte[] PublicKey = new byte[32];
        public int PeersCount;

        // Constructor that reads from memory
        public IoctlInterface(Pointer mem) {
            super(mem, ALIGN_DEFAULT);
            read(); // this reads memory into the structure fields
        }

        @Override
        protected List<String> getFieldOrder() {
            return Arrays.asList("Flags", "ListenPort", "PrivateKey", "PublicKey", "PeersCount");
        }
    }

    public static class SOCKADDR_INET extends Structure {
        public short si_family;
        public byte[] data = new byte[26]; // You can replace this with actual sockaddr structs later

        public SOCKADDR_INET() {
            super(ALIGN_DEFAULT);
        }

        @Override
        protected List<String> getFieldOrder() {
            return Arrays.asList("si_family", "data");
        }
    }

    public static class IoctlPeer extends Structure {
        public static final int SIZE = 136;

        public int Flags;
        public int Reserved;
        public byte[] PublicKey = new byte[32];
        public byte[] PresharedKey = new byte[32];
        public short PersistentKeepalive;
        public SOCKADDR_INET Endpoint = new SOCKADDR_INET();
        public long TxBytes;
        public long RxBytes;
        public long LastHandshake;
        public int AllowedIPsCount;

        public IoctlPeer(Pointer mem) {
            super(mem, ALIGN_DEFAULT);
            read(); // read values from memory into the structure
        }

        @Override
        protected List<String> getFieldOrder() {
            return Arrays.asList("Flags", "Reserved", "PublicKey", "PresharedKey", "PersistentKeepalive", "Endpoint",
                    "TxBytes", "RxBytes", "LastHandshake", "AllowedIPsCount");
        }
    }
}
