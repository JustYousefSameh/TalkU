package com.talku;

import com.talku.Controller.TalkUController;
import com.talku.Infrastruture.Wireguard.WireguardTunnel;
import com.talku.Presentation.TalkU;

public class Main {
    public static void main(String[] args) {
        if (args.length == 2 && args[0].equals("/service")) {
            WireguardTunnel.startWireguard(args[1]);
            return;
        }

        TalkU.main(args);
    }
}
