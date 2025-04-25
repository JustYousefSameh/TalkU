package com.talku.Presentation;

import java.net.URL;

import org.girod.javafx.svgimage.SVGImage;
import org.girod.javafx.svgimage.SVGLoader;

import javafx.animation.FillTransition;
import javafx.animation.Transition;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.scene.canvas.Canvas;
import javafx.scene.canvas.GraphicsContext;
import javafx.scene.layout.StackPane;
import javafx.scene.paint.Color;
import javafx.scene.paint.CycleMethod;
import javafx.scene.paint.LinearGradient;
import javafx.scene.paint.Stop;
import javafx.scene.shape.Rectangle;
import javafx.util.Duration;

public class GradientBackgroundWithImage extends StackPane {
    // A StackPane
    // Color gradient that changes color according to connection status
    // SVG Map Image
    private final Color disconnectedColor = Color.valueOf("#350000");
    private final Color connectingColor = Color.valueOf("#1D1A03");
    private final Color connectedColor = Color.valueOf("#082B09");

    private LinearGradient linearGradient = new LinearGradient(0, 0, 1, 1, true, CycleMethod.NO_CYCLE,
            new Stop(0, Color.BLACK), new Stop(1, disconnectedColor));

    private final Rectangle background = new Rectangle(450, 400);

    private Color startColor = disconnectedColor;
    private Color endColor;

    private Transition colorTransition = new Transition() {
        {
            setCycleDuration(Duration.millis(2150));
        }

        @Override
        protected void interpolate(double frac) {
            final Color newColor = startColor.interpolate(endColor, frac);
            linearGradient = new LinearGradient(0, 0, 1, 1, true, CycleMethod.NO_CYCLE,
                    new Stop[] { new Stop(0, Color.BLACK), new Stop(1, newColor) });

            background.setFill(linearGradient);
        }
    };

    public GradientBackgroundWithImage() {

        int width = 450;
        int height = 400;

        URL url = getClass().getResource("/world.svg");
        SVGImage worldImage = SVGLoader.load(url);

        // Starts as disconnected color
        background.setFill(linearGradient);

        colorTransition.setOnFinished((event) -> {
            startColor = linearGradient.getStops().get(1).getColor();
        });

        getChildren().addAll(background, worldImage);
    }

    void connected() {
        endColor = connectedColor;
        colorTransition.stop();
        colorTransition.play();
    }

    void connecting() {
        endColor = connectingColor;
        colorTransition.stop();
        colorTransition.play();
    }

    void disconnected() {
        endColor = disconnectedColor;
        colorTransition.stop();
        colorTransition.play();
    }

}
