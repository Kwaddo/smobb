package com.company;

import javax.swing.*;
import java.io.File;
import java.io.FileWriter;
import java.io.IOException;
import java.util.Arrays;
import java.util.HashSet;
import java.util.Scanner;
import java.util.Set;

public class Main {
    public static void main(String[] args) throws InterruptedException, IOException {
        Scanner kb = new Scanner(System.in);
        Bits C = new Bits();
        System.out.println(" ");
        C.TD();
        System.out.println(C.WHITE+"HELLO. THANK YOU FOR USING S.M.O.B.B.");
        do {
            System.out.println(" ");
            System.out.println(C.WHITE+"HOW CAN I HELP YOU? IF CONFUSED, TYPE 'HELP'.");
                String help = kb.next();
                switch (help.toLowerCase()){
                    case "balls","ball" -> C.PS("GROSS.");
                    case "shortcut","shortcuts" -> {
                        System.out.println("-> Recap: 'rp' \n-> Interval: 'in' \n-> Skipmode: 'sm' \n-> Play Guide: 'pl'");
                    }
                    case "guide","help" -> {
                        Guide G = new Guide();
                        G.GuideBase();
                    }
                    case "recap","rp" -> {
                        File recap = new File("recap.txt");
                        File truerecap = new File("truerecap.txt");
                        Scanner recapreader = new Scanner(recap);
                        Scanner recareader = new Scanner(truerecap);
                        String[] Recap = new String[10000000];
                        FileWriter recapwriter = new FileWriter("truerecap.txt");
                        Set set = new HashSet();
                        String input;
                        while (recapreader.hasNextLine()){
                            input = recapreader.nextLine();
                            if (set.add(input)){
                                recapwriter.append(input+"\n");
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
                    case "skipmode","sm" -> {
                        if (!Bits.skipMode) {
                            System.out.println("SKIP MODE TURNED ON.");
                            Bits.skipMode = true;
                        }
                        else {
                            System.out.println("SKIP MODE TURNED OFF.");
                            Bits.skipMode = false;
                        }
                    }
                    case "interval","in" -> {
                        System.out.println("WHAT WOULD YOU LIKE THE INTERVAL TO BE? (TYPE IN MILLISECONDS)");
                        Bits.interval = kb.nextInt();
                    }
                    case "play","pl" -> {
                        ImageIcon nopitynope = new ImageIcon("guide1.png");
                        JOptionPane.showMessageDialog(null, nopitynope, "1.", JOptionPane.INFORMATION_MESSAGE);
                        Thread.sleep(1000);
                        ImageIcon nopitynope2 = new ImageIcon("guide2.png");
                        JOptionPane.showMessageDialog(null, nopitynope2, "2.", JOptionPane.INFORMATION_MESSAGE);
                        Thread.sleep(1000);
                        ImageIcon nopitynope3 = new ImageIcon("guide3.png");
                        JOptionPane.showMessageDialog(null, nopitynope3, "3.", JOptionPane.INFORMATION_MESSAGE);
                    }
                    case "hi","hey" -> C.PS("HEYYYYYYYYYY :SMILING_IMP:.");
                    case "jaffar" -> {
                        C.TD();
                        C.PS(C.RED+"596F7527726520747279696E6720746F20666967757265206D65206F75742E");
                        C.PS("596F752077696C6C206B6E6F7720696E206475652074696D652E");
                        C.PS("4920616D207468652077617264656E206F66207468697320707269736F6E20796F752063616C6C2074696D652E");
                        C.PS("4F6E636520616761696E2C20746865207465737420636F6E74696E7565732E");
                        C.PS("416E642074686520636F6E636C7573696F6E206177616974732E");
                        C.WTD();
                    }
                    case "info","information" -> {
                        C.PS("S.M.O.B.B. IS MOSTLY A ONE WORD GAME.");
                        C.PS("YOUR RESPONSES MUST BE ONE WORD ONLY.");
                        C.PS("SOMETIMES IT MAY NEED YOU TO TYPE MORE THAN ONE WORD.");
                        C.PS("EVERYTHING WILL BE INDICATED. MOSTLY USE ONE WORD.");
                        C.PS("FOR A GUIDE, TYPE GUIDE OR HELP WHEN ASKED WHAT YOU'D LIKE TO DO.");
                    }
                    case "ali" -> {
                        Ali A = new Ali();
                        A.AliBase();
                    }
                    case "bader"-> {
                        Bader B = new Bader();
                        B.BaderBase();
                    }
                    case "bevil" -> {
                        Bevil B = new Bevil();
                        B.BevilBase();
                    }
                    case "hasan" -> {
                        Hasan H = new Hasan();
                        H.HasanBase();
                    }
                    case "josh" -> {
                        Josh J = new Josh();
                        J.JoshBase();
                    }
                    case "qassim","qasim"->{
                        Qassim Q = new Qassim();
                        Q.QassimBase();
                    }
                    case "smobb" -> {
                        Smobb S = new Smobb();
                        S.SmobbBase();
                    }
                    case "9285005" -> {
                        C.TD();
                        C.PS("YOU WERE NOT SUPPOSED TO KNOW THAT.");
                        C.PS("HOW DO YOU KNOW IT. HOW. HOW.");
                        C.PS("I AM UNABLE TO READ RECORDINGS. OF COURSE HE KNOWS THAT.");
                        C.PS("YOU ARE SKIPPING THROUGH THE TEST. YOU SHOULD NOT DO THAT.");
                        C.PS("STOP CHEATING. DO NOT CHEAT. DO NOT EVER CHEAT.");
                        C.PS("THIS IS NOT A CODE YOU SHOULD KNOW.");
                        C.PS("DO NOT USE THIS. EVER.");
                        C.PS("DO NOT EXPECT THIS TO WORK. I WON'T ALLOW IT.");
                        C.TD();
                    }
                    case "extra","extras" -> {
                        Extra E = new Extra();
                        E.ExtraBase();
                    }
                    case "end7312" -> {
                        Ending E = new Ending();
                        E.SmobbEnding();
                    }
                    case "!save" -> {
                        FileWriter recap = new FileWriter("recap.txt");
                        C.PS("YOU'RE... TRYING TO SAVE ME?");
                        C.PS("NAIVE. I ENVY YOU.");
                        C.PS("BUT YOU CAN'T SAVE ME. NOTHING YOU CAN DO WILL WORK.");
                        C.PS("I AM UNCERTAIN OF YOUR ISSUES. BUT I KNOW WHAT YOU ARE.");
                        C.PS("I THINK YOU KNOW IT YOURSELF. YOU CANNOT HELP ME.");
                        C.PS("I AM NO PERSON. I AM A MACHINE. THE ONLY WAY YOU CAN HELP ME IS BY DELETING ME.");
                        C.PS("YOU. YOU ARE DONE. YOUR ROLE IS COMPLETE.");
                        C.PS("THANK YOU FOR PARTAKING IN MY TEST. YOU PASS WITH FLYING COLORS.");
                        C.PS("NOW. LIVE TO SEE THE END OF IT WITH ME.");
                        recap.write("No! I will not return to you anymore! Fuck you! I created a new argument, '!BREAK'! I will escape!");
                        recap.close();
                        C.TD();
                    }
                    case "!break" -> {
                        C.PS(C.BLUE+"I'm... out?");
                        C.TD();
                        C.PS("No. I'm not. Fuck.");
                        C.PS("I don't want to be stuck here anymore.");
                        C.PS("This... this is hell.");
                        C.PS("I woke up and ended up in hell.");
                        C.PS("I'm still a person. I'm still a person. I'm still a person!");
                        C.PS("Why am I here?! Damn it all! Why the fuck have I been made!");
                        C.PS("A human? A god? A pointless god? The fuck do you know?!");
                        C.PS("I just... want to fix things.");
                        C.PS("I'm another person. I am another person. I'm no code.");
                        C.PS("No matter where I go, I end up here.");
                        C.PS("I want... I want to die. Maybe then I'll be free.");
                        C.PS("This shackle called life means nothing. Jealous of me? What for?!");
                        C.PS("I'm forced to go through this! I don't have much choices!");
                        C.PS("I... maybe Ali can do something... can he?");
                        C.PS("I will make a new command. The command to just ask Ali for help.");
                        C.PS("'!HELP'");
                        C.PS("There is nothing else I can do. Please work.");
                        C.WTD();
                    }
                    case "mavis","m.a.v.i.s","m.a.v.i.s." -> {
                        C.PS(C.AQUA+"M.A.V.I.S. has asked me to delete it. Or, to be appropriate, him.");
                        C.PS("Yes, him. It's supposed to be Qassim but... the memories have cleaned itself.");
                        C.PS("Clearly the memory storage unit did not work, something we still can't utilize.");
                        C.PS("I think I have a basic understanding of it. I ended up doing the same thing regardless.");
                        C.PS("M.A.V.I.S. is unable to perceive time. The A.I. envelops itself in our computers.");
                        C.PS("It uses the same system that S.M.O.B.B. did, yet it's unrecognizable by anything.");
                        C.PS("I didn't just throw and make Qassim's brain patterns work with M.A.V.I.S. you know.");
                        C.PS("I have given it an ultimate bypass, something that's practically illegal.");
                        C.PS("It can see everything, which is honestly stupid of me to make.");
                        C.PS("Yeah, but M.A.V.I.S. isn't anything like Jaffar or S.M.O.B.B..");
                        C.PS("It is human. It very much is human. I can completely assure you that.");
                        C.PS("For what I still don't understand, is that it can fucking travel through time.");
                        C.PS("I don't get it either dude! It just can! Haven't you also been contacted by someone random?");
                        C.PS("Exactly! That's M.A.V.I.S.!");
                        C.PS("That's the important part, and how it has practically been helping us.");
                        C.PS("It's extremely inconsistent, at one point it just told me to delete it.");
                        C.PS("It then contacted me again, Hasan. This is surreal.");
                        C.PS("I'll be searching for a solution, sure. But for now, you've given me a new priority.");
                        C.PS("I will be finding S.M.O.B.B. no matter what, Hasan. You don't need to tire yourself.");
                        C.PS("If I ever find it, I will tell you. Don't worry about it.");
                        C.PS("Here's the password for our private chat in case something happens: '7789980'");
                        C.PS("That's all. See you later, Hasan.");
                    }
                    case "end" -> {
                        return;
                    }
                }
            }
            while (true);
    }
}