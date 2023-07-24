package com.company;
import java.io.IOException;
import java.util.Scanner;
import java.io.FileWriter;
public class Bevil {
    Bits C = new Bits();
    public void BevilBase() throws InterruptedException, IOException {
        Scanner kb = new Scanner(System.in);
        FileWriter recap = new FileWriter("recap.txt");
        do {
            System.out.println(" ");
            System.out.println(C.GOLD+"Hey I'm about to take a shower, what do you need?");
            String bevil = kb.next();
            switch (bevil.toLowerCase()) {
                case "!save" -> {
                    C.PS("Hm? Who are you? What the fuck?");
                    C.PS("Uh, sure. Whatever.");
                    C.PS("What do you want exactly?");
                    C.PS("Help Hasan? Why? What'll happen?");
                    C.PS("What kind of mistake did he even make? Or will make?");
                    C.PS("Fucking hell, what! Alright, I better stop him then.");
                    C.PS("Or try to. I don't even know if you're right or wrong.");
                    C.PS("Thanks, I guess. Bye.");
                }
                case "7789980" ->{
                    C.TD();
                    C.PS("Ali. This account won't exist by the time you receive this message.");
                    C.PS("I'm sorry for taking so long to respond, yet it's still what you expected.");
                    C.PS("We accounted deaths into this. Such a thing was beyond our control.");
                    C.PS("I hate how I couldn't do more at first. I really hate it.");
                    C.PS("I... I feel like an idiot. I can't believe I've ditched you.");
                    C.PS("Yes I was closest with Bader, but you two were still my friends.");
                    C.PS("You would like me to clear up some things... yeah?");
                    C.PS("At first, Bader and Qassim were constantly fighting. You remember this.");
                    C.PS("But Bader didn't tell you why they were fighting. It wasn't even because of S.M.O.B.B..");
                    C.PS("I have no clue how they were able to hide it for so long, but both of them were working at MORP.");
                    C.PS("Bader was still a new employee, Qassim has been working there for a while by now.");
                    C.PS("Qassim had to go to his cousin's wedding, so he asked Bader to do watch over his important project.");
                    C.PS("I don't know what this project was, but it had something to do with brains I think?");
                    C.PS("Bader complied, telling him that he can go. He shouldn't have.");
                    C.PS("Bader, knowing nothing about Qassim's work, closed the terminal once he was about to leave.");
                    C.PS("The terminal shut down everything in an instant, and Bader quickly realized what he had done.");
                    C.PS("The system was supposed to constantly work. Even a second of shutdown will mean that it will be destroyed.");
                    C.PS("Keep in mind this was way back. You should remember such issues starting two years ago.");
                    C.PS("Qassim had a backup system, but that's only under failsafe. This was a direct shutdown.");
                    C.PS("Everything disappeared. Bader couldn't even get himself tell Qassim.");
                    C.PS("Unluckily for him, Qassim enters just to check. He then saw everything that happened.");
                    C.PS("I don't know what happened after, Bader just told me they fought so much during that hour.");
                    C.PS("Soon after, with multiple complaints from Qassim, Bader got fired.");
                    C.PS("He was eventually able to find something else, but in the end this is why both him and Qassim fought.");
                    C.PS("Bader's an idiot. Qassim's an asshole. You're incompetent. I'm a coward.");
                    C.PS("I wanted to deal with all of this no more. I really just wanted to drop everything.");
                    C.PS("It wasn't worth it. Of course it wasn't.");
                    C.PS("I'm sorry. I'm... sorry. I have literally no other words.");
                    C.PS("I didn't want to ditch you when you needed me the most. I feel as if it's already too late to return.");
                    C.PS("So... goodbye, Ali. I hope the best for you.");
                    C.TD();
                }
                case "7549009" -> {
                    C.PS("That's... the password, right?");
                    C.PS("What's the point. Why are you telling me?");
                    C.PS("Josh, I don't want this. Why are you sending it to me?");
                    C.PS("I have tried this entire time not to involve myself.");
                    C.PS("Problems are increasing because pretty much everyone's children!");
                    C.PS("Does no one understand that doing nothing would keep the peace?!");
                    C.PS("Josh, you're putting yourself in the middle of this.");
                    C.PS("It is not worth it. Not even a little bit.");
                    C.PS("Listen. Find an actual job, and never contact us again.");
                    C.PS("This pays nothing, and you're not doing much.");
                    C.PS("Stop. Shut up. I'm not deciding for you, make the decision yourself.");
                    C.PS("We are not friends, but I'm considerate. Nobody deserves involvement.");
                    C.PS("Go, and don't come back. I won't talk to you again.");
                }
                case "9285005" -> {
                    C.PS("Ali?! Where the fuck have you been?!");
                    C.PS("No, no, no! Shut the fuck up!");
                    C.PS("I don't care what Jaffar is doing to Josh! I don't care about this crap anymore!");
                    C.PS("There are now two dead people! I'm not gonna fucking do shit!");
                    C.PS("Do what you like but I don't care.");
                    C.PS("Ali. Listen and do not reply in any way.");
                    C.PS("I was closest with Bader. Before our last argument, we were in a call almost everyday.");
                    C.PS("Our last argument involved Qassim. Whom he himself told me he killed.");
                    C.PS("All I've been trying to do is stop you from hacking the files.");
                    C.PS("And you didn't stop. You just made a bad situation worse.");
                    C.PS("Now Bader deserved prison. Not death. Even if it was manipulation.");
                    C.PS("You are all people I met online. I'm done talking to you.");
                    C.PS("I'm not gonna get involved in a problem that is frankly getting worse.");
                    C.PS("Deal with it how you'd like, you fucking left us.");
                    C.PS("I'm done, don't talk to me again.");
                }
                case "jaffar","ja3ffar", "ja3ffer"-> {
                    C.PS("Yeah, I figured it out.");
                    C.PS("This is a secure line, right? It shouldn't be able to breach.");
                    C.PS("The language is Hexadecimal. No wonder we can't understand it.");
                    C.PS("Yeah, we can use a converter. A simple one at a website.");
                    C.PS("Man you know I'm not gonna code a converter and you won't either.");
                    C.PS("Yeah, I already found a website too. Want the link?");
                    C.PS("https://www.rapidtables.com/convert/number/hex-to-ascii.html");
                    recap.write("Jaffar speaks using Hexadecimal language. Use this link to convert: https://www.rapidtables.com/convert/number/hex-to-ascii.html.");
                    recap.close();
                    C.PS("I'll play tomorrow, I promise. I don't have any space for today.");
                    C.PS("What do you mean bro, I gotta take a shower then pray then work out then do my work.");
                    C.PS("No I don't have time dummy, see ya.");
                }
                case "taradaloop"-> {
                    C.TD();
                    C.PS("Yeah, I'm right here Bader. I figured it out.");
                    C.PS("You know how when you open a picture with notepad, there's some weird shit you can't understand?");
                    C.PS("Since Jaffar is attracted to the pictures straight from the source, couldn't we just use it as a trap?");
                    C.PS("Then let's just remove some part of the pictures' code.");
                    C.PS("I've figured out something, I'm calling it the Tarada Loop.");
                    C.PS("Just listen, just listen. I'll make a trap using wall codes and trick him with the entrance.");
                    C.PS("It has to work, there's nothing else we can use.");
                }
                case "josh"-> {
                    C.PS("Isn't that the guy Bader hired as a joke?");
                    C.PS("He's barely even being paid anything, I have no clue why he's still here.");
                    C.PS("I'm not even sure if he's gonna involve himself in our situation. I hope not.");
                    C.PS("He might want to leave soon enough.");
                    C.PS("Ali, you've been trying to hack into the files yeah?");
                    C.PS("Just, don't do that. It's pointless.");
                    C.PS("Learn to just ignore it, this time I'm serious.");
                    C.PS("There's a reason it's still here, I'll tell you later ok?");
                    C.PS("Please bear with me for now. Please.");
                }
                case "breach" -> {
                    C.PS("Hasan sent me this executable file. I have no idea what it does.");
                    C.PS("Yeah, it sounds pretty important, there's even a note attached.");
                    C.PS("It basically says that this program is meant to prevent any breaches.");
                    C.PS("I'm assuming breaches from the AI that literally just disappeared into thin air?");
                    C.PS("Bader, use your brain. We need this. It's called 'prevent.exe'.");
                    C.PS("What's funny is that it's just a single line of code. Wait I'll send it to you one sec.");
                    C.PS("'<---------->'");
                    recap.write("The breach prevention code is '<---------->'.");
                    recap.close();
                    System.out.print("Yeah I have no idea what that means. I'm sure we just run the executable when there's a brea");
                    System.out.println(C.WHITE+"--._-...__----");
                    String prevent = kb.next();
                    if (prevent.equals("<---------->")) {
                        C.PS(C.GOLD+"Yeah, that noise! That's the breach!");
                        C.PS("]That's all though. Goodnight baby girl.");
                    } else {
                        C.TD();
                        C.PS(C.RED+"49276D206C61746520616761696E21");
                        C.PS("44616D6E69742120");
                        C.WTD();
                    }
                }
                case "ali" -> {
                    C.PS("What the fuck are you talking about?");
                    C.PS("Yeah of course I don't believe you! There's no way that could have happened!");
                    C.PS("He just exploded? The Tarada Files self destructed? Shut the fuck up dude.");
                    C.PS("You don't joke about this kind of shit.");
                    C.PS("That can't be the only explanation, did you even go to his apartment?");
                    C.PS("You did? Send me a picture right now.");
                    C.PS("You're trolling in some way, there's just no way.");
                    C.PS("Fine, I'll believe you. It's just very sudden, and very unrealistic.");
                    C.PS("I'm sure you don't have a reason to lie about this, my bad.");
                    C.PS("Just listen to me. I'm going to explain everything, we need as many help as we can get.");
                    C.PS("Just listen.");
                    C.WTD();
                }
                case "hasan" -> {
                    C.PS("Yeah, Bader's acting really stupid now.");
                    C.PS("I really don't blame Ali for getting angry, anyone would.");
                    C.PS("It's only getting worse and worse, especially after Qassim's death");
                    C.PS("You have to talk to Ali, he seems to listen to you now.");
                    C.PS("Yeah, I know the password. Don't worry.");
                    C.PS("It's '7549009', you can tell li if you want.");
                    recap.write("The password for the Tarada files is '7549009'.");
                    recap.close();
                    C.PS("Bader doesn't know that I know the password, don't worry.");
                    C.PS("At least if Ali's gonna hack let him get it over with as soon as possible.");
                    System.out.print("It's 3AM I gotta go to sleep, goodnight");
                    C.PS(C.WHITE+".--.__.--____-...-_--");
                    String prevent = kb.next();
                    if (prevent.equals("<---------->")) {
                        C.PS(C.GOLD+"That screeching noise! Wait, did it hear us?!");
                        C.PS("Goddamnit, we need to stop it from hacking the files!");
                        C.PS("I'll tell Josh to surround the file with breach preventions.");
                        C.PS("Yeah, he's still awake for sure.");
                        C.PS("Goddamnit, such a bad time dude!");
                    } else {
                        C.TD();
                        C.PS(C.RED+"54686572652120546865726520697420697321");
                        C.PS("5468657920636F756C646E27742068696465207468652070617373776F72642E");
                        C.PS("4E6F7720746865792063616E27742068696465207468652066696C657320666F72206C6F6E672E");
                        C.PS("492077696E21");
                        C.PS("4F682C2072696768742E20596F7527726520686572652E");
                        C.PS("497420646F65736E2774206D617474657220616E797761792E");
                        C.PS("54686520746573742077696C6C20636F6E74696E75652E");
                        C.WTD();
                    }
                }
                case "bader" -> {
                    C.PS("Hasan, I told Bader that I told Ali.");
                    C.PS("Yeah, I know I didn't have to take the blame. I did anyway.");
                    C.PS("It feels as if Bader's plan is falling apart.");
                    C.PS("But it's not, Ali's still gonna hack the fucking files.");
                    C.PS("Is there nothing we can really do?");
                    C.PS("Why were we kept in the middle of this fucking problem man.");
                    C.PS("Yeah I'm done as well, I just hope that by hacking the files it'll be over with.");
                    C.PS("They could have stopped this drama ages ago, but they didn't.");
                    C.PS("They're both still children in this way, alone they're mature.");
                    C.PS("Alright, goodnight.");
                }
                case "end"-> {
                    return;
                }
            }
        }while (true);
    }
}