package com.company;

public class Bits {
    public static boolean skipMode = false;
    public static int interval = 4250;
    public String BLACK = "\u001B[30m";
    public String RED = "\u001B[31m"; /* Jaffar */
    public String GREEN = "\u001B[32m";
    public String YELLOW = "\u001B[33m";
    public String BLUE = "\u001B[34m"; /* Mavis */
    public String PURPLE = "\u001B[35m"; /* Hasan */
    public String CYAN = "\u001B[36m";
    public String AQUA = "\u001B[38;5;14m"; /* Ali */
    public String WHITE = "\u001B[38;5;15m"; /* S.M.O.B.B. */
    public String SALMON = "\u001B[38;5;209m"; /* Bader */
    public String SALMON2 = "\u001B[38;5;210m"; /* Bader under mind control */
    public String GOLD = "\u001B[38;5;220m"; /* Bevil */
    public String LIGHTGREEN = "\u001B[38;5;119m"; /* Josh */
    public String SILVER = "\u001B[38;5;7m"; /* Qassim */
    public String AQUAMARINE = "\u001B[38;5;122m"; /* Josh under mind control */
    public void PS(String S) throws InterruptedException {
        System.out.println(S);
        if (!skipMode) {
            Thread.sleep(interval);
        } else {
            Thread.sleep(0);
        }
    }
    public void TD() throws InterruptedException {
        System.out.print(".");Thread.sleep(2000);System.out.print(".");Thread.sleep(2000);System.out.print(".");Thread.sleep(2000);
    }
    public void WTD() throws InterruptedException {
        Bits C = new Bits();
        System.out.print(C.WHITE+".");Thread.sleep(2000);System.out.print(".");Thread.sleep(2000);System.out.print(".");Thread.sleep(2000);
    }
}