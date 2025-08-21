package com.talku;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

import com.sun.jna.Native;
import com.sun.jna.platform.win32.Kernel32;
import com.sun.jna.platform.win32.Tlhelp32;
import com.sun.jna.platform.win32.User32;
import com.sun.jna.platform.win32.WinDef;
import com.sun.jna.platform.win32.WinDef.HWND;
import com.sun.jna.platform.win32.WinNT;
import com.sun.jna.platform.win32.WinUser;
import com.talku.Infrastruture.WinService.InstanceManager;
import com.talku.Infrastruture.Wireguard.WireguardTunnel;
import com.talku.Presentation.TalkU;

import javafx.application.Platform;

public class Main {
    public static void main(String[] args) {
        if (args.length == 2 && args[0].equals("/service")) {
            WireguardTunnel.startWireguard(args[1]);
            return;
        }

        if (!InstanceManager.getInstance().isPrimaryInstance(() -> {

            Platform.runLater(() -> {
                TalkU.showStage();
            });

        })) {
            System.out.println("Another instance is running. Sending SHOW command and exiting...");
            InstanceManager.getInstance().sendShowMessage();
            return;
        }

        TalkU.main(args);
    }

}
