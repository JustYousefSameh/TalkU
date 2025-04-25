package com.talku.Presentation;

import javafx.animation.FadeTransition;
import javafx.animation.ScaleTransition;
import javafx.scene.shape.Rectangle;
import javafx.scene.paint.Color;
import javafx.util.Duration;

public class ScalingCircle extends Rectangle {

    private ScaleTransition circleAnim = new ScaleTransition(Duration.millis(2000), this);
    private FadeTransition circleFade = new FadeTransition(Duration.millis(2000), this);

    ScalingCircle() {
        setWidth(180);
        setHeight(80);
        setArcHeight(80);
        setArcWidth(80);
        setStroke(Color.valueOf("#082B09"));
        setStrokeWidth(15);
        setOpacity(0);
        setFill(Color.TRANSPARENT);

        circleFade.setInterpolator(PresentationUtils.easeInOutBack);
        circleFade.setDelay(Duration.millis(250));

        circleAnim.setDelay(Duration.millis(250));
    }

    public void animate() {
        circleFade.setFromValue(0.6);
        circleFade.setToValue(0);
        circleAnim.setFromX(1);
        circleAnim.setFromY(1);
        circleAnim.setToX(20);
        circleAnim.setToY(20);
        circleFade.play();
        circleAnim.play();
    }
}
