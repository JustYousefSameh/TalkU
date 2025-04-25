package com.talku.Infrastruture.Wireguard;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;
import com.sun.jna.WString;
import com.sun.jna.ptr.IntByReference;

public interface WireGuardLibrary extends Library {
    // Load the DLL — it must be in your PATH or working directory
    WireGuardLibrary INSTANCE = Native.load("wireguard", WireGuardLibrary.class);

    // Equivalent to: IntPtr WireGuardOpenAdapter(LPCWSTR name);
    Pointer WireGuardOpenAdapter(WString name);

    // Equivalent to: void WireGuardCloseAdapter(IntPtr adapter);
    void WireGuardCloseAdapter(Pointer adapter);

    // Equivalent to: bool WireGuardGetConfiguration(IntPtr adapter, byte[] iface, ref UInt32 bytes);
    boolean WireGuardGetConfiguration(Pointer adapter, byte[] iface, IntByReference bytes);
}