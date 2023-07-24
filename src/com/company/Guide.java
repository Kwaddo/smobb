package com.company;
import javax.swing.*;
import java.io.*;
import java.util.*;

public class Guide {
    Bits C = new Bits();
    public void GuideBase() throws InterruptedException, IOException {
        System.out.println(C.WHITE+"WHAT DO YOU NEED HELP WITH?");
        do {
            Scanner kb = new Scanner(System.in);
            System.out.println(" ");
            System.out.println("YOUR OPTIONS ARE: NAMES/GAME/SETTINGS/RECAP/END.");
            String guide = kb.next();
            Guide G = new Guide();
            switch (guide.toLowerCase()) {
                case "names", "name" -> G.NameGuide();
                case "game","play" -> G.PlayGuide();
                case "recap" -> G.RecapGuide();
                case "settings" -> G.Settings();
                case "end" -> {
                    System.out.println("WHILE IN A LOOP, YOU CAN EXIT BY TYPING 'END'. YOU WILL NOW EXIT THE LOOP.");
                    return;
                }
            }
        } while (true);
    }
    public void Settings() throws InterruptedException {
        Scanner kb = new Scanner(System.in);
        do{
            System.out.println("OPTIONS ARE: INTERVAL/SKIPMODE/END.");
            String setting = kb.next();
            switch (setting.toLowerCase()){
                case "interval" -> {
                    System.out.println("WHAT WOULD YOU LIKE THE INTERVAL TO BE? (TYPE IN MILLISECONDS)");
                    Bits.interval = kb.nextInt();
                }
                case "skipmode" -> {
                    if (!Bits.skipMode) {
                        System.out.println("SKIP MODE TURNED ON.");
                        Bits.skipMode = true;
                    }
                    else {
                        System.out.println("SKIP MODE TURNED OFF.");
                        Bits.skipMode = false;
                    }
                }
                case "end" -> {
                    return;
                }
            }
        } while (true);

    }
    public void NameGuide() throws InterruptedException {
        do {
            Scanner kb = new Scanner(System.in);
            System.out.println("WHO WOULD YOU LIKE TO KNOW ABOUT?");
            System.out.println("TYPE 'OPTIONS/CHOICES' FOR OPTIONS.");
            String name = kb.next();
            switch (name.toLowerCase()) {
                case "ali" -> {
                    C.PS(C.AQUA+"ALI MOHAMMED HASAN ALHALWACHI.");
                    C.PS(C.WHITE+"WELL KNOWN SOFTWARE ENGINEER. KNOWN FOR CREATING MAID-IT, AND MORE PRIVATE CREATIONS.");
                    C.PS("THE CENTER.");
                    C.PS("THE TARADA FILES SURROUND HIM.");
                }
                case "bader" -> {
                    C.PS(C.SALMON+"BADER MAHMOOD ABDULKHALAF.");
                    C.PS(C.WHITE+"COMPUTER SCIENCE MAJOR, PROJECTS CURRENTLY UNKNOWN.");
                    C.PS("I DO NOT UNDERSTAND ANYTHING SURROUNDING HIM, BUT HE IS THE CREATOR OF THE TARADA FILES.");
                    C.PS("THE TARADA FILES ARE EVERYTHING SURROUNDING ALI. PICTURES, SCREENSHOTS, VIDEOS, MEMES. EVERYTHING.");
                    C.PS("NOTHING MORE IS KNOWN.");
                }
                case "bevil" -> {
                    C.PS(C.GOLD+"BEVIL, OR ALSO KNOWN AS GNU8305. REAL IDENTITY UNKNOWN.");
                    C.PS(C.WHITE+"HELPING BADER WITH HIS WORK. MAJOR UNKNOWN.");
                    C.PS("NOTHING IS KNOWN ABOUT HIM. NO INFORMATION AVAILABLE.");
                }
                case "hasan" -> {
                    C.PS(C.PURPLE+"HASAN AHMED TARADA.");
                    C.PS(C.WHITE+"GENIUS A.I. CREATOR, KNOWN FOR [REDATCTED].");
                    C.PS("ONE COULD BLAME HIM FOR EVERYTHING THAT IS HAPPENING.");
                    C.PS("BUT HE IS NOT THE ONLY ONE AT FAULT.");
                    C.PS("THE ONLY WAY TO DESCRIBE HIM, IS A RAT TRAPPED BY ITS OWN CAGE.");
                }
                case "josh" -> {
                    C.PS(C.LIGHTGREEN+"JOSH. NO MORE, IT IS ONLY JOSH.");
                    C.PS(C.WHITE+"HUGE FAN OF MOVIES, VIDEO GAMES, and TV SHOWS.");
                    C.PS("CURRENTLY JOBLESS, YET SOMEHOW BEING PAID. PECULIAR.");
                }
                case "qassim" -> {
                    C.PS(C.SILVER+"QASSIM MOHAMMED HAMEED. UNDERGROUND SCIENTIST.");
                    C.PS(C.WHITE+"WORKS AT A COMPANY CALLED MORP. SHORT FOR [REDACTED].");
                    C.PS("PROJECTS INVOLVE BRAIN TRANSFERRAL PROGRAM.");
                }
                case "smobb" -> {
                    C.PS("ME.");
                }
                case "options","choices", "choice" -> {
                    System.out.println("YOUR OPTIONS ARE: ALI/BADER/BEVIL/JOSH/HASAN/QASSIM/SMOBB/END.");
                }
                case "end" -> {
                    return;
                }
            }
        } while (true);
    }
    public void PlayGuide() throws InterruptedException {
        ImageIcon nopitynope = new ImageIcon("guide1.png");
        JOptionPane.showMessageDialog(null, nopitynope, "1.", JOptionPane.INFORMATION_MESSAGE);
        Thread.sleep(1000);
        ImageIcon nopitynope2 = new ImageIcon("guide2.png");
        JOptionPane.showMessageDialog(null, nopitynope2, "2.", JOptionPane.INFORMATION_MESSAGE);
        Thread.sleep(1000);
        ImageIcon nopitynope3 = new ImageIcon("guide3.png");
        JOptionPane.showMessageDialog(null, nopitynope3, "3.", JOptionPane.INFORMATION_MESSAGE);
    }
    public void RecapGuide() throws InterruptedException, IOException {
        Scanner kb = new Scanner(System.in);
        File recap = new File("recap.txt");
        File truerecap = new File("truerecap.txt");
        Scanner recapreader = new Scanner(recap);
        Scanner recareader = new Scanner(truerecap);
        String[] Recap = new String[10000000];
        System.out.println("YOUR OPTIONS ARE: OPENRECAP/CLEARRECAP.");
        String unrecap = kb.next();
        switch (unrecap.toLowerCase()) {
            case "openrecap" -> {
                FileWriter recapwriter = new FileWriter("truerecap.txt");
                Set set = new HashSet();
                String input = null;
                while (recapreader.hasNextLine()){
                    input = recapreader.nextLine();
                    if (set.add(input)){
                        recapwriter.append(input + "\n");
                    }
                }
                recapwriter.flush();
                while (recareader.hasNextLine()) {
                    for (int i = 0; i < Recap.length; i++) {
                        Recap = new String[]{recareader.nextLine()};
                        C.PS(Arrays.toString(Recap));
                    }
                }
            }
            case "clearrecap" -> {
                FileWriter fw = new FileWriter("recap.txt", false);
                PrintWriter pw = new PrintWriter(fw, false);
                pw.flush();
                pw.close();
                fw.close();
                C.PS("Recap files cleared!");
            }
        }
    }
}