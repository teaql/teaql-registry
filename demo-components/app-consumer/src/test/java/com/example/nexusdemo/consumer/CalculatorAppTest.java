package com.example.nexusdemo.consumer;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class CalculatorAppTest {
    @Test
    void testCalculation() {
        String result = CalculatorApp.calculateMessage(10, 20);
        assertEquals("Sum: 30, Product: 200", result);
    }
}
