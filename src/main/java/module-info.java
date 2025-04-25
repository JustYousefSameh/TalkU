module talku {
    // JavaFX
    requires javafx.base;
    requires javafx.controls;
    requires javafx.graphics;
    requires javafx.fxml;
    requires javafx.swing;

    // Java SE
    requires java.desktop;
    requires java.datatransfer;
    requires java.sql;
    requires java.xml;
    requires java.net.http;

    // Third-party libs
    requires com.fasterxml.jackson.databind;
    requires com.fasterxml.jackson.core;
    requires com.fasterxml.jackson.annotation;
    requires io.vavr;
    requires de.jensd.fx.glyphs.fontawesome;
    requires org.girod.javafx.svgimage;
    requires de.jensd.fx.glyphs.commons;
    requires de.jensd.fx.glyphs.materialicons;
    requires junique;
    requires com.sun.jna;
    requires com.sun.jna.platform;

    // Export your own packages
    exports com.talku.Presentation;
    exports com.talku.Controller;
    exports com.talku.Infrastruture.VpnConfig;
    exports com.talku.Infrastruture.WinService;
    exports com.talku.Infrastruture.Wireguard;
    exports com.talku.Utils;

}
