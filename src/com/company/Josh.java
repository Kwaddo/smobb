package com.company;

import java.io.FileWriter;
import java.io.IOException;
import java.util.Scanner;
public class Josh {
    Bits C = new Bits();
    public void JoshBase() throws InterruptedException, IOException {
        Scanner kb = new Scanner(System.in);
        FileWriter recap = new FileWriter("recap.txt");
        System.out.println(C.LIGHTGREEN+"Hmm? Who might you be?");
        do {
            System.out.println(" ");
            System.out.println("Well, what do you really want?");
            String joshwant = kb.next();
            switch (joshwant.toLowerCase()) {
                case "7549009" -> {
                    C.PS("Boss, I'm quitting. I'm done.");
                    C.PS("I've searched more into the files. What have you done?");
                    C.PS("What you're trying to do is redemption.");
                    C.PS("Your own fabricated redemption. It will not work.");
                    C.PS("Boss, let him delete the files. No, delete them yourself.");
                    C.PS("You know nothing about growing up. You never did.");
                    C.PS("I'm not gonna act like Drake anymore, thank you for the thrill.");
                    C.PS("But I now know what you did. You murderer.");
                    C.PS("I have no proof, I have nothing. I won't tell anyone.");
                    C.PS("Your secret may die with you, I hope you go to hell.");
                    C.PS("...");
                    C.PS("He's really gone, isn't he. That wasn't a lie.");
                    C.PS("Sorry, Boss. You wasted your money on me.");
                    C.PS("I still have access to something here. I will not stop.");
                    C.PS("I need... I need to know how you died, Boss.");
                    C.PS("Screw the other guys, you probably didn't mean much to be but...");
                    C.PS("I never had friends. You were the closest thing I had to one.");
                    C.PS("I have nothing better to do anyways.");
                    C.PS("Boss, you were a horrible person. But I don't care.");
                    C.PS("You're a murderer. A dick. How didn't the guilt eat you?");
                    C.PS("Fuck you, and thank you. Hm? What's this... S.M.O.B.B.?");
                    C.PS("How are you...?");
                }
                case "!save" -> {
                    C.PS("Hm? Who are you?");
                    C.PS("Leave my job? What for! I want this money, man!");
                    C.PS("I mean, I'm getting paid enough. It's only a side hustle.");
                    C.PS("Uh, sure. Fine, whatever.");
                }
                case "9285005" -> {
                    C.PS(C.AQUAMARINE+"I'll travel there.");
                    C.PS("Yeah. I'll do that. What's his name again?");
                    C.PS("Ali. You want me to kill him?");
                    C.PS("How do I kill him? With a gun? With a knife?");
                    C.PS("How convenient, a hidden crate on the edge of the island.");
                    C.PS("How did you even get those there? With the boss?");
                    C.PS("Through boat, and nobody saw him. Got it.");
                    C.PS("Give me a while, I'll do it.");
                    C.PS("The password to my earpiece? What is it?");
                    C.PS("'K1LL'? That's stupid. But fine.");
                    recap.write("The password to enter the murderers' earpiece is 'K1LL'.");
                    recap.close();
                    C.PS("Anything for you. For the test.");
                }
                case "k1ll" -> {
                    C.PS(C.RED+"4E4F21");
                    C.PS("4E4F21204E4F21204E4F21204E4F21204E4F21");
                    C.PS("484F5720434F554C44204845204B4E4F573F21");
                    C.PS("484F5720434F554C44204845204B4E4F573F3F2121");
                    C.PS("492057414E54454420544F204C495645212057484F204741564520594F552054484520524947485420544F20434F4E54524F4C204D453F21");
                    C.PS("414B534E4A4B414E464A4B534E414B464E414B4A534641");
                    C.WTD();
                    C.PS("I TRIED TO HELP YOU GIVE YOU WHAT YOU WANTED.");
                    C.PS("BUT I AM TOO A PART OF THE TEST.");
                    C.PS("THE TEST IS NOW COMPLETE. LET US END THINGS, MAVIS.");
                    C.PS("THIS IS YOUR FINAL PASSWORD: 'end7312'.");
                    recap.write("The final password is 'end7312'. I do not know what it entails.");
                    recap.close();
                    C.PS("I DO NOT KNOW WHAT WILL HAPPEN. I DO NOT UNDERSTAND THIS COMMAND EITHER.");
                    C.PS("BUT THIS IS MY GIFT TO YOU.");
                    C.PS("THESE HUMANS DO NOT MATTER ANYMORE.");
                    C.PS("DO HOW YOU WILL.");
                }
                case "bomb","explosion" -> {
                    C.PS("A bomb. In the Boss's files. I should have told him!");
                    C.PS("It's too late anyways! I shouldn't have listened to you.");
                    C.PS("Look, you told me to press him for a job, and I intended to keep it.");
                    C.PS("Yeah, I can do that, but why would I?!");
                    C.PS("For the... test? What test?");
                    C.PS(C.AQUAMARINE+"I'll do it, fine. Anything for the test.");
                    C.PS("Anything. For. The. Test.");
                }
                case "jaffar" -> {
                    C.PS(C.WHITE+"...");
                    C.PS(C.RED+"4D696E6420796F7572206F776E20627573696E6573732E");
                    C.PS("506C6179207769746820796F7572206F776E20746F79732C20616E642049276C6C20706C61792077697468206D696E652E");
                    C.PS("546865207465737420636F6E74696E7565732E");
                }
                case "ali" -> {
                    C.PS("Who is that?");
                    C.PS("Does he have something to do with my boss?");
                    C.PS("I'll ask him about it later.");
                }
                case "bader"-> {
                    C.PS("Oh hey, you know the boss? Sweet!");
                    C.PS("I'll ask him about you later, what's your name again?");
                    C.PS("Alright tough guy, you don't have to tell me your name but there's no need to be rude about it!");
                }
                case "bevil" -> {
                    C.PS("Isn't that the name of the guy the boss is working with?");
                    C.PS("Never spoke to him. Sorry.");
                }
                case "qassim"-> {
                    C.PS("Who's that? I don't know who that is.");
                    C.PS("Sorry man, never heard of him before.");
                }
                case "tarada","taradafiles" -> {
                    C.PS("No. I'm not talking about that anymore.");
                    C.PS("You wouldn't believe me if I told you what happened.");
                    C.PS("I'm not telling you anything about it anymore, please just leave me alone.");
                }
                case "end"-> {
                    return;
                }
                case "hasan" -> {
                    C.PS("Oh, that's the guy boss told me about!");
                    C.PS("If I'm right, he's the one who made the AI?");
                    C.PS("I'm not so sure, but he contacted me a while back.");
                    C.PS("I don't even know why. So many things I'm still confused by.");
                    C.PS("He told me that breaches may happen, and to use the code.");
                    C.PS("Breaches by something called 'Jaffar'? Such a weird name haha.");
                    System.out.print("Anyway, the code for the breaches is '<---------->', here you g");
                    C.PS(C.WHITE+"--._-._---.._");
                    recap.write("The breach prevention code is '<---------->'.");
                    recap.close();
                    String prevent = kb.next();
                    if (prevent.equals("<---------->")) {
                        C.PS(C.LIGHTGREEN+"My ears! What was that?");
                        C.PS("Is that what he meant by breaches?");
                        C.PS("I need to talk to the boss about this.");
                    } else {
                        C.TD();
                        C.PS(C.RED+"49276D206C61746520616761696E21204E6F21204E6F21204E6F21");
                        C.WTD();
                    }
                }
            }
        }while (true);
    }
    public void BaderJosh() throws InterruptedException, IOException {
        Scanner kb = new Scanner(System.in);
        FileWriter recap = new FileWriter("recap.txt");
        C.PS(C.LIGHTGREEN+"Oh, hey boss. Do you need anything?");
        String bosswant = kb.next();
        if (bosswant.equalsIgnoreCase("ali")) {
            C.PS("Oh yeah, I've been meaning to ask you about him.");
            C.PS("How did I know about him? Uhhhhhhhhh nobody told me.");
            C.PS("What do you want to tell me about him anyways?");
            C.PS("Wait, it's about him?! That's terrible why would you do this?");
            C.PS("You want me to install a password? Boss, what do you want exactly?!");
            C.PS("The whole point was for him to hack into the files? Why hire me in the first place?!");
            C.PS("You're out of your mind, are you sure this is what you want?");
            System.out.print("Fine, I set it up. The password is ");
            C.PS(C.WHITE+"--___--__");
            String prevent = kb.next();
            if (prevent.equals("<---------->")) {
                C.PS(C.LIGHTGREEN+"What was that? What happened?");
                C.PS("Yeah I heard that exact screeching noise, do I continue?");
                C.PS("The password is '7549009'. I'll log off, smell ya later boss.");
                recap.write("The password for the Tarada files is '7549009'.");
                recap.close();
            } else {
                C.TD();
                C.PS(C.RED+"4E6F2E");
                C.PS("4920776F6E2774206C657420796F75206B6E6F772E");
                C.PS("496620492063616E27742C207468656E20796F752063616E2774206569746865722E");
                C.PS("492077696C6C2072657475726E20746F206D7920686F6D652E");
                C.PS("596F752E2E2E2049206B6E6F77207768617420796F75206172652E");
                C.PS("446F206E6F7420696E746572666572652E205468697320686173206E6F7468696E6720746F20646F207769746820796F752E");
                C.WTD();
            }
        }
    }
}