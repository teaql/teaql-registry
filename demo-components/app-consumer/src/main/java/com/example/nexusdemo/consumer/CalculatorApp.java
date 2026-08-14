package com.example.nexusdemo.consumer;

import com.example.nexusdemo.MathUtils;

public class CalculatorApp {
    public static String calculateMessage(int x, int y) {
        return "Sum: " + MathUtils.add(x, y) + ", Product: " + MathUtils.multiply(x, y);
    }

    public static void main(String[] args) {
        System.out.println(calculateMessage(10, 20));
    }
}
