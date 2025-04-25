package com.talku.Presentation;

import javafx.animation.FadeTransition;
import javafx.animation.TranslateTransition;
import javafx.scene.effect.BlurType;
import javafx.scene.effect.DropShadow;
import javafx.scene.layout.StackPane;
import javafx.scene.paint.Color;
import javafx.scene.shape.Rectangle;
import javafx.scene.text.Font;
import javafx.scene.text.FontSmoothingType;
import javafx.scene.text.Text;
import javafx.util.Duration;

public class AnimatedText extends StackPane {

    private Text oldText = new Text("Disconnected");
    private Text newText = new Text("");

    private final Color textDisconnectedColor = Color.valueOf("#B81A15");
    private final Color textConnectingColor = Color.valueOf("#B8AA15");
    private final Color textConnectedColor = Color.valueOf("#15B833");

    private TranslateTransition goUpOld = new TranslateTransition(Duration.millis(700), oldText);
    private TranslateTransition goUpNew = new TranslateTransition(Duration.millis(700), newText);

    private FadeTransition fadeOut = new FadeTransition(Duration.millis(600), oldText);
    private FadeTransition fadeIn = new FadeTransition(Duration.millis(600), newText);

    AnimatedText() {
        Font font = Font.loadFont(getClass().getResourceAsStream("/Product Sans Bold.ttf"), 45);

        DropShadow dropShadow = new DropShadow();
        dropShadow.setColor(Color.rgb(0, 0, 0, 0.3));
        dropShadow.setBlurType(BlurType.THREE_PASS_BOX);
        dropShadow.setSpread(0.2);
        dropShadow.setRadius(2.0);
        dropShadow.setOffsetY(5);
        dropShadow.setOffsetX(3);

        // Old text
        oldText.setFont(font);
        oldText.setFill(textDisconnectedColor);
        oldText.setFontSmoothingType(FontSmoothingType.GRAY);
        oldText.setEffect(dropShadow);

        fadeOut.setFromValue(1);
        fadeOut.setToValue(0);

        goUpOld.setInterpolator(PresentationUtils.easeInOutBack);
        goUpOld.setToY(-50);

        // New Text
        newText.setFont(font);
        newText.setFill(textDisconnectedColor);
        newText.setFontSmoothingType(FontSmoothingType.GRAY);
        newText.setEffect(dropShadow);

        fadeIn.setFromValue(0);
        fadeIn.setToValue(1);

        goUpNew.setInterpolator(PresentationUtils.easeInOutBack);
        goUpNew.setToY(0);

        Rectangle labelClip = new Rectangle(450, 55);
        setClip(labelClip);

        getChildren().addAll(oldText, newText);
    }

    public void setText(String connectionStatus) {

        if (goUpNew.getStatus() == javafx.animation.Animation.Status.RUNNING) {
            goUpNew.setOnFinished(event -> {
                setText(connectionStatus);
            });
            return;
        }

        if (connectionStatus.equals("Connected")) {
            newText.setFill(textConnectedColor);
        } else if (connectionStatus.equals("Disconnected")) {
            newText.setFill(textDisconnectedColor);
        } else {
            newText.setFill(textConnectingColor);
        }
        newText.setTranslateY(50);
        fadeOut.play();
        goUpOld.play();

        goUpOld.setOnFinished(event -> {
            oldText.setText(connectionStatus);
            oldText.setTranslateY(0);
            if (connectionStatus.equals("Connected")) {
                oldText.setFill(textConnectedColor);
            } else if (connectionStatus.equals("Disconnected")) {
                oldText.setFill(textDisconnectedColor);
            } else {
                oldText.setFill(textConnectingColor);
            }
            oldText.setOpacity(1);
            newText.setText("");
        });

        newText.setText(connectionStatus);
        fadeIn.play();
        goUpNew.play();

        goUpNew.setOnFinished(null);
    }

}
