package com.talku.Infrastruture.Wireguard;

import com.talku.Infrastruture.WinService.Win32Service;

public class WireGuardTunnelService extends Win32Service {

    public WireGuardTunnelService() {
        super("WireGuardTunnel$TalkU");
    }

}
