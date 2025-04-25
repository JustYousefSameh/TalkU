package com.talku.Presentation;

import java.awt.Desktop;
import java.net.URI;

import javafx.animation.TranslateTransition;
import javafx.geometry.Pos;
import javafx.scene.Cursor;
import javafx.scene.control.Button;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.layout.Background;
import javafx.scene.layout.StackPane;
import javafx.util.Duration;

public class SocialButton extends Button {

    public static SocialButton discordButton() {
        SocialButton button = new SocialButton("https://discord.gg/mph7jETDv9", "/discord.png");
        button.setTranslateX(-5);
        button.setTranslateY(30);
        return button;
    }

    public static SocialButton githubButton() {
        SocialButton button = new SocialButton("https://github.com/JustYousefSameh/TalkU", "/github.png");
        button.setTranslateX(40);
        button.setTranslateY(30);
        return button;
    }

    private String socialUrl;

    private TranslateTransition justTransition = new TranslateTransition(Duration.millis(300), this);

    private SocialButton(String url, String iconPath) {
        Image image = new Image(iconPath);
        ImageView imageView = new ImageView(image);
        imageView.setFitWidth(32);
        imageView.setPreserveRatio(true);

        socialUrl = url;

        setPrefHeight(60);
        setPrefWidth(60);
        setBackground(Background.EMPTY);

        StackPane.setAlignment(this, Pos.BOTTOM_LEFT);

        justTransition.setInterpolator(PresentationUtils.easeInOutBack);

        setGraphic(imageView);

        setOnAction(event -> {
            try {
                Desktop.getDesktop().browse(new URI(socialUrl));
            } catch (Exception e) {

            }
        });

        setOnMouseEntered(event -> {
            justTransition.setToY(10);
            justTransition.stop();
            justTransition.play();
            setCursor(Cursor.HAND);

        });

        setOnMouseExited(event -> {
            justTransition.setToY(30);
            justTransition.stop();
            justTransition.play();
            setCursor(Cursor.DEFAULT);
        });

    }
}
