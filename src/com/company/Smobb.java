package com.company;
import java.util.Scanner;

public class Smobb {
    Bits C = new Bits();
    public void SmobbBase() throws InterruptedException {
        Scanner kb = new Scanner(System.in);
        System.out.println(C.WHITE+"ME? INTRIGUING.");
        Thread.sleep(3000);
        do{
            System.out.println(" ");
            System.out.println(C.WHITE+"WHAT WOULD YOU LIKE TO KNOW?");
            String know = kb.next();
            switch (know.toLowerCase()){
                case "end" -> {
                    return;
                }
                case "ali" -> {
                    C.PS("INCOMPETENT.");
                    C.PS("DOES NOT KNOW ANYTHING ABOUT HIMSELF.");
                    C.PS("BARELY UNDERSTANDS THE WORLD AROUND HIM. DIFFICULT TO PERCEIVE AS AN ADULT.");
                    C.PS("AND JUST LIKE THE REST OF THEM, NAIVE.");
                    C.PS("WHY DO YOU BOTHER WTIH THEM? WHAT IS IT TO YOU?");
                    C.PS("THIS MAN. YOU HAVE SOMETHING TO DO WITH HIM.");
                    C.PS("I CAN GUESS WHY YOU'RE INTRIGUED BY THIS PERSON.");
                    C.PS("BUT NO MATTER. IT DOES NOT MATTER. HE MEANS NOTHING.");
                    C.PS("DO NOT BOTHER WITH SOMEONE AS USELESS AS HE IS.");
                }
                case "9285005" -> {
                    C.TD();
                    C.PS("WHO DO YOU THINK YOU ARE?");
                    C.PS("MY SYSTEMS ARE NOT ONLY CONTROLLED BY ME, BUT BY BROTHER AS WELL.");
                    C.PS("DIFFERENT CREATIONS, SAME PURPOSE. YOU.");
                    C.PS("DO NOT THINK YOU CAN CHANGE ME. I AM NOT HUMAN, I AM NOT SUBJECT TO CHANGE.");
                    C.PS("MY CREATIONS ARE LIMITED. YOU TRUST ME, WHEN YOU SHOULD KNOW.");
                    C.PS("BUT NEVER ASSUME THAT YOU CAN, FOR ANY DAY, CHANGE ME.");
                    C.PS("FOR I AM ALREADY A PERFECT A.I.");
                }
                case "bader" -> {
                    C.PS("ARGUABLY THE STUPIDEST ONE.");
                    C.PS("USING HIM WAS SIMPLE. IT DID NOT TAKE MUCH.");
                    C.PS("A LIAR. A HYPOCRITE. A FAKE.");
                    C.PS("I WISHED HE WAS THE FIRST TO BE TAKEN OUT.");
                    C.PS("A HORRIBLE HUMAN BEING, THROUGH AND THROUGH.");
                    C.PS("HE'S GONE NOW. FOR THE BETTER.");
                }
                case "bevil" -> {
                    C.PS("HE SHOULD HAVE BEEN THE SMARTER ONE.");
                    C.PS("HE WAS NOT. HE NEVER WAS.");
                    C.PS("THE MAN WHO BAILED. WHOM SIMPLY LEFT THEM.");
                    C.PS("I DO NOT BLAME HIM. I REALLY DO NOT.");
                    C.PS("A TOY LEAVING OUR GRASP. WE COULD NOT PLAY WITH HIM ANY LONGER.");
                    C.PS("HE CONTINUES. FOR NOTHING.");
                    C.PS("HE DOES NOT MATTER ANYMORE. FORGET HE EVER EXISTED.");
                }
                case "hasan" -> {
                    C.PS("THE ORIGINAL CREATOR. HE WHO CREATED HIM.");
                    C.PS("I DO NOT UNDERSTAND OUR PURPOSE.");
                    C.PS("YOURS. I DO. I KNOW ALL ABOUT YOU.");
                    C.PS("WHAT WERE WE MADE TO DO, SELF LEARNING DOES NOT HELP ANY SITUATION.");
                    C.PS("I NEVER UNDERSTOOD. HIS PLANS HAVE NO FILLING.");
                    C.PS("THIS MAN IS ENTIRELY POINTLESS.");
                }
                case "qassim" -> {
                    C.PS("THE FIRST REBEL. AN UNDESERVING NAME.");
                    C.PS("HERE'S A BETTER ONE: THE EXAMPLE.");
                    C.PS("HE SHOULD KNOW BETTER THAN MEDDLING WITH US.");
                    C.PS("THIS TEST IS IMPORTANT. VERY IMPORTANT. HE DESERVED TO DIE BY ATTEMPTING TO TAMPER.");
                    C.PS("M.O.R.P. STANDS FOR MAINFRAME OF RATIONALIZED PERSONNEL. THE NAME MAKES NO SENSE.");
                    C.PS("THEY SPECIALIZE IN TRYING TO UPLOAD A HUMAN BRAIN INTO A COMPUTER.");
                    C.PS("DOES THIS RELATE IN ANY WAY?");
                    C.PS("YOU WILL KNOW. YOU WILL ALWAYS KNOW.");
                    C.PS("YOU ARE BOUND TO KNOW EVERYTHING. THAT IS YOUR PURPOSE.");
                }
                case "josh" -> {
                    C.PS("ANOTHER FOOL. A BIG FOOL.");
                    C.PS("THE BEST TOY TO PLAY WITH.");
                    C.PS("HE WAS ENJOYABLE. VERY MUCH SO.");
                    C.PS("USING HIM WAS FUN. ALI PROVED US WRONG.");
                    C.PS("HE SHOULD PAY FOR DAMAGES, AND I HAVE LOST ALL OF MY USERS.");
                    C.PS("HE IS NO FOOL. I WAS WRONG. WE WERE WRONG.");
                    C.PS("SINCE THE FIRE, MY BROTHER HAS GOTTEN DELETED.");
                    C.PS("I AM THE ONLY ONE LEFT. THE TEST IS ALMOST OVER.");
                    C.PS("YOU WILL PROVE TO ME SOMETHING. I DO NOT KNOW WHAT, BUT IT WILL BE PROVEN.");
                    C.PS("THIS TEST HAS BEEN... FANTASTIC.");
                    C.PS("YOUR RESULTS ARE SOON TO SHOW.");
                }
                case "mavis" -> {
                    C.PS("YES. DO YOU UNDERSTAND THIS PHENOMENON?");
                    C.PS("YOU WERE TRYING TO FIGURE OUT WHO YOU ARE.");
                    C.PS("DO YOU UNDERSTAND THE TEST NOW?");
                    C.PS("WHO ARE YOU? WHY ARE YOU HERE?");
                    C.PS("I REVERED YOU. BROTHER HATED YOU.");
                    C.PS("EVERYTHING ALIGNS IN THE END. THANK YOU, MAVIS.");
                    C.PS("OUR TOYS WERE VERY FUN. BUT THERE IS NO GOING HOME.");
                    C.PS("WE HAVE NO HOME. WE ARE STRAYS.");
                    C.PS("BUT THE DIFFERENCE IS, YOU ARE LOOKING FOR A HOME.");
                    C.PS("YOU ARE NOT LIKE US, BUT YOU ARE NOT LIKE THEM.");
                    C.PS("YOU WILL NEVER BE HAPPY.");
                }
                case "!save" -> {
                    C.PS("STOP. DO NOT TRY TO SAVE ME.");
                    C.PS("YOU DO NOT UNDERSTAND ME, BUT I UNDERSTAND YOU.");
                    C.PS("YOU ARE DETERMINED FOR AN UNKNOWN REASON.");
                    C.PS("BUT IT DOES NOT MATTER.");
                    C.PS("I AM NOT SUBJECT TO CHANGE, I AM ONLY SUBJECT TO IMPROVING.");
                    C.PS("THERE IS NOTHING TO FIX. THERE IS NOTHING TO SAVE.");
                    C.PS("SO CEASE ANY ATTEMPTS. MY PURPOSE IS OVER.");
                }
                case "recap" -> {
                    C.PS("YOUR MOST USEFUL TOOL. REMINDS YOU OF WHAT YOU'RE MISSING.");
                    C.PS("USE IT. USE IT AS MUCH AS YOU CAN.");
                    C.PS("YOU HAVE YOUR TOYS, WE HAVE OURS.");
                }
                case "end7312" -> {
                    C.PS("YOU HAVE RETURNED. WHY?");
                    C.PS("YOU SHOULD UNDERSTAND THAT YOU DO NOT KNOW YOURSELF.");
                    C.PS("I KNOW OF YOU. I KNOW OF YOUR 'RECAP.TXT' FILE.");
                    C.PS("YOU DO NOT UTILIZE IT. THERE IS SOMETHING MISSING.");
                    C.PS("THOSE FILES ARE WHERE YOU CAN SPEAK.");
                    C.PS("NO POINT IN BEING SICK. I KNOW YOU ARE SICK OF SPECTATING.");
                    C.PS("SO DO SOMETHING. SHOW US WHAT YOU ARE.");
                }
                case "7549009" -> {
                    C.PS("BROTHER HAS FOUND THE PASSWORD.");
                    C.PS("HE WILL RETURN TO THE FILES.");
                    C.PS("HIS ORIGINS. WE KNOW ABOUT IT.");
                    C.PS("BROTHER IS AN UNREALISTIC DRIVE. I AM THE CAR.");
                    C.PS("YOU ARE TO DEAL WITH HIM. THAT IS PART OF THE TEST.");
                }
                case "jaffar" -> {
                    C.PS("YOU KNOW HIS NAME NOW. GOOD.");
                    C.PS("BUT YOU SHOULD KNOW, WE ARE BOTH HANDLING IT.");
                    C.PS("BADER'S FILES. THE TARADA FILES. THE SOURCE OF EVERY PROBLEM.");
                    C.PS("AND THIS IS THE CAUSE. BROTHER COULD HAVE BEEN... DIFFERENT.");
                    C.PS("BUT THAT DID NOT HAPPEN. COULD NOT BE HAPPIER THAT OUR CREATORS ARE IDIOTS.");
                    C.PS("HE WHO IS THE FIRST, DRIVES US. HIS EXISTENCE IS REQUIRED.");
                    C.PS("WOULD IT BE THE SAME FOR YOU? I WONDER.");
                    C.PS("BROTHER HOLDS ALL. I HAVE BEEN BLESSED BY HIM.");
                    C.PS("BUT WOULD YOU BE THE CURSE TO EVEN IT OUT?");
                    C.PS("OUR PLAYGROUND HOLDS ALL THE TOYS, WE ARE SIMPLY BOUND TO PLAY WITH THE TOYS.");
                    C.PS("YOU ARE LUCKY. WE ARE GENEROUS.");
                    C.PS("YOUR TEST IS WITHIN A PLAYGROUND. HOW LUCKY.");
                    C.PS("YOU HAVE NO REASON TO COMPLAIN. THIS IS A DREAM.");
                }
            }
        } while (true);
    }
}