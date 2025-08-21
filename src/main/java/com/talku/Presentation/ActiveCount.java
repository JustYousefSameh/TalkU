package com.talku.Presentation;

import java.util.Random;
import java.util.Timer;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;

import com.talku.Infrastruture.VpnConfig.ConfigHandler;

import javafx.animation.ScaleTransition;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.effect.DropShadow;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Region;
import javafx.scene.paint.Color;
import javafx.scene.shape.Circle;
import javafx.scene.text.Font;
import javafx.scene.text.Text;
import javafx.util.Duration;

public class ActiveCount extends HBox {

    private final Text label;
    private final Circle dot;

    private ScheduledExecutorService scheduler;

    private int count;
    private int userEffectiveCount;

    private ScheduledExecutorService userEffectiveCountScheduler;

    public ActiveCount() {
        super(6); // spacing between dot and text

        label = new Text("....");

        // Layout
        setAlignment(Pos.CENTER);
        this.setPadding(new Insets(0, 12, 0, 12));
        this.setMaxSize(Region.USE_PREF_SIZE, 30);

        // Dot with glow
        this.dot = new Circle(5, Color.LIMEGREEN);
        DropShadow glow = new DropShadow(7, Color.LIMEGREEN);
        glow.setSpread(0.4);
        dot.setEffect(glow);

        // Text label
        Font font = Font.loadFont(getClass().getResourceAsStream("/Roboto-Regular.ttf"), 13);
        label.setFont(font);
        label.setFill(Color.WHITE);
        label.setOpacity(0.45);

        scheduler = Executors.newScheduledThreadPool(1);

        scheduler.scheduleAtFixedRate(() -> {
            var result = ConfigHandler.getConnectedUsersCount();
            if (result.isRight()) {
                count = result.get();
                setUserEffectiveCount(0);
                label.setText(String.valueOf(count + userEffectiveCount));
            }

        }, 0, 120, TimeUnit.SECONDS);

        this.getChildren().addAll(dot, label);

    }

    public void close() {
        scheduler.shutdownNow();
    }

    public void setUserEffectiveCount(int connectedOrDisconnected) {
        userEffectiveCount = connectedOrDisconnected;

        switch (connectedOrDisconnected) {
        case 0:
            if (userEffectiveCountScheduler != null && !userEffectiveCountScheduler.isShutdown()) {
                userEffectiveCountScheduler.shutdownNow();
                userEffectiveCountScheduler = null;
            }
            break;

        case 1:
            if (userEffectiveCountScheduler != null && !userEffectiveCountScheduler.isShutdown()) {
                userEffectiveCountScheduler.shutdownNow();
                userEffectiveCountScheduler = null;
            }

            userEffectiveCountScheduler = Executors.newSingleThreadScheduledExecutor();
            userEffectiveCountScheduler.schedule(() -> {
                System.out.println("2 minutes passed, resetting active count to original number");
                setUserEffectiveCount(0);
            }, 2, TimeUnit.MINUTES);

            break;
        }

        label.setText(String.valueOf(count + userEffectiveCount));

    }
}
