package com.talku.Presentation;

import java.awt.Desktop;
import java.net.URI;

import javafx.animation.ScaleTransition;
import javafx.animation.TranslateTransition;
import javafx.css.Size;
import javafx.geometry.Pos;
import javafx.scene.Cursor;
import javafx.scene.control.Button;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.layout.Background;
import javafx.scene.layout.StackPane;
import javafx.scene.transform.Scale;
import javafx.util.Duration;

public class SocialButton extends Button {

    public static SocialButton discordButton() {
        SocialButton button = new SocialButton("https://discord.gg/mph7jETDv9", "/discord.png", 32);
        button.setTranslateX(-5);
        button.setTranslateY(5);
        return button;
    }

    public static SocialButton githubButton() {
        SocialButton button = new SocialButton("https://github.com/JustYousefSameh/TalkU", "/github.png", 30);
        button.setTranslateX(40);
        button.setTranslateY(4);
        return button;
    }

    private String socialUrl;

    // Used scale animation instead of the old one cuz people thought it was bugged for some reason >:(
    private ScaleTransition socialTransition = new ScaleTransition(Duration.millis(300), this);

    private SocialButton(String url, String iconPath, int size) {
        Image image = new Image(iconPath);
        ImageView imageView = new ImageView(image);
        imageView.setFitWidth(size);
        imageView.setPreserveRatio(true);

        socialUrl = url;

        setPrefHeight(60);
        setPrefWidth(60);
        setBackground(Background.EMPTY);

        StackPane.setAlignment(this, Pos.BOTTOM_LEFT);

        socialTransition.setInterpolator(PresentationUtils.easeInOutBack);

        setGraphic(imageView);

        setOnAction(event -> {
            try {
                Desktop.getDesktop().browse(new URI(socialUrl));
            } catch (Exception e) {

            }
        });

        setOnMouseEntered(event -> {
            socialTransition.setToX(1.2);
            socialTransition.setToY(1.2);
            socialTransition.stop();
            socialTransition.play();
            setCursor(Cursor.HAND);

        });

        setOnMouseExited(event -> {
            socialTransition.setToX(1);
            socialTransition.setToY(1);
            socialTransition.stop();
            socialTransition.play();
            setCursor(Cursor.DEFAULT);
        });

    }
}
