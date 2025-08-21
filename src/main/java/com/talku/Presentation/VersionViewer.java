package com.talku.Presentation;

import javax.sound.sampled.Clip;

import javafx.animation.Interpolator;
import javafx.animation.TranslateTransition;
import javafx.scene.layout.StackPane;
import javafx.scene.paint.Color;
import javafx.scene.shape.Rectangle;
import javafx.scene.text.Font;
import javafx.scene.text.Text;
import javafx.util.Duration;

public class VersionViewer extends Text {

    public VersionViewer(String version) {
        setText(version);
        Font font = Font.loadFont(getClass().getResourceAsStream("/Roboto-Regular.ttf"), 11);
        setFont(font);
        setFill(Color.WHITE);
        setOpacity(0.3);
    }

}
