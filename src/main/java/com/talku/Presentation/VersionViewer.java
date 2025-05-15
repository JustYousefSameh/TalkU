package com.talku.Presentation;

import javafx.scene.paint.Color;
import javafx.scene.text.Font;
import javafx.scene.text.Text;

public class VersionViewer extends Text {

    public VersionViewer(String version) {
        setText(version);
        Font font = Font.loadFont(getClass().getResourceAsStream("/Roboto-Regular.ttf"), 13);
        setFont(font);
        setFill(Color.WHITE);
        setOpacity(0.3);
        setTranslateX(-10);
        setTranslateY(-10);
    }

}
