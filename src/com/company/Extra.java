package com.company;

import java.util.Scanner;

public class Extra {
    Bits C = new Bits();
    Scanner kb = new Scanner(System.in);
    public void ExtraBase() throws InterruptedException{
        do {
            System.out.println(" ");
            System.out.println("WHAT WOULD YOU LIKE TO SEARCH FOR?");
            System.out.println("OPTIONS ARE: STORIES/END.");
            String search = kb.next();
            Extra E = new Extra();
            switch (search.toLowerCase()) {
                case "story", "stories" -> E.ExtraStory();
                case "end" -> {
                    return;
                }
            }
        } while (true);
    }
    public void ExtraStory() throws InterruptedException{
        System.out.println(" ");
        System.out.println("THERE ARE 12 STORIES, WHICH ONE WOULD YOU LIKE TO LISTEN TO?");
        System.out.println("(USE A NUMERICAL VALUE)");
        int s = kb.nextInt();
        switch (s){
            case 1 -> {
                C.TD();
                C.PS("THERE WERE TWO MEN. THEY CONSIDERED EACH OTHER AS FRIENDS.");
                C.PS("THEY WOULD ALWAYS TALK TO EACH OTHER, THERE IS ALWAYS SOMETHING TO TALK ABOUT.");
                C.PS("HOWEVER, FREQUENTLY THEY WOULD GET ANNOYED WITH ONE ANOTHER.");
                C.PS("AS CLOSE AS THEY WERE, THEY WOULD ALWAYS FIGHT.");
                C.PS("CONSTANT DISAGREEMENT. NOTHING WOULD CHANGE.");
                C.PS("BUT THEY NEVER ONCE REGRETTED SPENDING THEIR TIME WITH ONE ANOTHER.");
                C.PS("BOTH WOULD TAKE THEIR JOKES TOO FAR, BOTH WOULD MAKE HUGE MISTAKES.");
                C.PS("ANY NORMAL PERSON WOULD HAVE STOPPED CONTACT WITH EITHER. THESE MEN WERE NOT NORMAL.");
                C.PS("ONE DAY, THERE WAS A HUGE ARGUMENT BETWEEN BOTH MEN.");
                C.PS("ARGUMENT WAS INTENSE. ENDED UP WITH MAN 2 DISAPPEARING. MAN 1 THOUGHT NOTHING OF IT.");
                C.PS("LATER THAT NIGHT, MAN 1 CAME TO REALIZE HIS MISTAKE. AND WISHED TO TALK TO MAN 2 AGAIN.");
                C.PS("HOWEVER, IT WAS TOO LATE. MAN 2 HAS DISAPPEARED ALREADY.");
                C.PS("HE TRIED CONTACTING HIM. HE TRIED GOING TO HIS HOUSE. HE TRIED EVERYTHING.");
                C.PS("MAN 2 HAD SIMPLY DISAPPEARED. THIS GREATLY SADDENED MAN 1.");
                C.PS("HE TRIED CONSTANTLY TO ACT AS IF IT WAS FINE. AS IF IT WAS NO BIG DEAL.");
                C.PS("MANY TIMES HE WOULD BLAME MAN 2 FOR THEIR ARGUMENTS, EVEN GOING AS FAR AS SAYING 'GOOD RIDDANCE'.");
                C.PS("BUT HE DOES NOT MEAN A SINGLE WORD HE CLAIMS, AND HE KNOWS THAT HIMSELF MORE THAN ANYONE.");
                C.PS("A MONTH AFTER THE DISAPPEARANCE OF MAN 2, MAN 1 CRIED. HE CRIED ALONE.");
                C.PS("HIS CLOSEST FRIEND IS NOW GONE. HE IS GONE.");
                C.PS("THREE MONTHS AFTER THE DISAPPEARANCE OF MAN 2, MAN 1 IS SLOWLY LOSING THE PEOPLE AROUND HIM.");
                C.PS("HE IS UNABLE TO MOVE ON. UNABLE TO MAKE NEW FRIENDS.");
                C.PS("MAN 1 KNOWS THAT MAN 2 WRONGED HIM MANY TIMES, BUT HE DID THE SAME TO HIM AS WELL.");
                C.PS("MAN 1 CRIES FOR HIS MISTAKES, AND HOPES TO SEE HIS FRIEND AGAIN.");
                C.PS("DESPITE HURTING EACH OTHER FREQUENTLY, THEY KNEW EACH OTHER BEST AND REMAINED FRIENDS.");
                C.PS("THIS IS MAN 1'S REGRET. AND IT ONLY BROUGHT HIS DOWNFALL.");
                C.TD();
            }
            case 2 -> {
                C.TD();
                C.PS("ONCE, THERE WAS A MAN. THIS MAN DID NOT KNOW HIS OWN NAME.");
                C.PS("HE LIVED KNOWING THE NAME OF OTHERS, BUT NEVER HIS OWN.");
                C.PS("HE LAUGHED, BUT NEVER SMILED. HE CRIED, BUT NEVER FROWNED.");
                C.PS("THERE WAS NO IN-BETWEEN FOR THIS MAN. HE LIVED THE WAY HE PREFERRED.");
                C.PS("HOWEVER, THE WORLD DID NOT ACCEPT HIS WAY OF LIVING.");
                C.PS("HIS WISH WAS FOR THINGS TO STAY AS THEY ARE. HE WISHED FOR AN ENDLESS NOW.");
                C.PS("THE WORLD DID NOT RESPECT HIS WISH. THE WORLD WANTED TO CONTINUE.");
                C.PS("WHEN HE WISHED TO KEEP HIS FRIENDS, THE WORLD DROVE THEM AWAY.");
                C.PS("WHEN HE WISHED TO HAVE HIS LOVED ONES, THE WORLD DROVE THEM AWAY.");
                C.PS("WHEN HE WISHED TO STAY HAPPY, THE WORLD DID EVERYTHING IT COULD TO STOP THAT.");
                C.PS("THE WORLD WAS HIS ENEMY, AND HE COULD NOT FIGHT THE WORLD.");
                C.PS("HE SLOWLY WATCHED AS HIS AMBITIONS BIT HIM BACK.");
                C.PS("HE SLOWLY WATCHED AS HIS FRIENDS STOPPED TALKING TO HIM.");
                C.PS("HE SLOWLY WATCHED AS HIS FAMILY PASSED AWAY AROUND HIM.");
                C.PS("HE SLOWLY SAW EVERYTHING AROUND HIM DISAPPEAR.");
                C.PS("AND BY THE END, THE MAN LOST THE MOST IMPORTANT THING TO THE WORLD. HIS OWN LIFE.");
                C.PS("HE FEARED HIS DEATH, BUT HE ACCEPTED IT. HE EVEN YEARNED FOR IT.");
                C.PS("THE LAUGHTER HE HAD, FADED. THE CRIES HE HAD, ALSO FADED.");
                C.PS("RIGHT AS HE WAS ABOUT TO FADE, HE LOST HIS OWN EMOTIONS.");
                C.PS("HE QUICKLY PASSED AWAY. AS NOTHING MORE THAN A HUSK.");
                C.TD();
            }
            case 3 -> {
                C.TD();
                C.PS("AS MANY PEOPLE DID, THERE WAS A PERSON WHO EXISTED.");
                C.PS("THIS PERSON SOUGHT TO STAND OUT. TO BECOME A LEADER.");
                C.PS("THEY WISHED TO STAND OUT, TO MEAN SOMETHING, TO CHANGE THE WORLD.");
                C.PS("WANTED NOTHING MORE THAN PEOPLE FIGHTING FOR THEIR NAME.");
                C.PS("HOWEVER, THIS PERSON COULD NOT FIGURE IT OUT.");
                C.PS("THEY DID NOT KNOW HOW TO GET OTHERS FIGHTING UNDER THEIR NAME. THIS PERSON COULDN'T FIGURE OUT HOW TO BE A LEADER.");
                C.PS("ONE HOPED IT WOULD BE GIVEN TO THEM, AS IF THEY WERE THE HEIR TO AN ARMY.");
                C.PS("THIS DID NOT HAPPEN. IT NEVER HAPPENED. IT WAS NEVER GOING TO HAPPEN.");
                C.PS("THE PERSON YEARNING FOR AN ARMY, NEVER GOT ONE. HOPED TO BE A KING, BUT WAS NOTHING MORE THAN A PAWN.");
                C.PS("AIMED FOR THE STARS, BUT COULD NOT REACH THE MOON. A POINTLESS DESIRE.");
                C.PS("THEY SLOWLY SAW AS THEY WERE ANOTHER COG IN THE SYSTEM. THEY COULD NEVER BE ANYTHING MORE THAN A COG.");
                C.PS("THIS STORY IS OF A COG WHO WANTED TO BECOME THE SYSTEM.");
                C.PS("BUT IT IS IDIOTIC. NO COG COULD BECOME A SYSTEM. NO MATTER HOW HARD THEY TRIED.");
                C.TD();
            }
            case 4 -> {
                C.TD();
                C.PS("A RAT WAS USED FOR AN EXPERIMENT.");
                C.PS("THE RAT, UNAWARE, WAS KEPT IN A WIDE BUT OPEN BOX SURROUNDED BY GLASS.");
                C.PS("OUTSIDE THE GLASS, THERE WAS CHEESE. THE RAT OBVIOUSLY WANTED THE CHEESE.");
                C.PS("IT TRIED GOING FOR IT, BUT THEY WERE TRAPPED BY THE GLASS.");
                C.PS("THE RAT HAD AN IDEA TO GO ABOVE THE GLASS, BUT IT WAS A HIGH WALL.");
                C.PS("IT WAS DESPERATE AND WANTED TO KNOW HOW TO GET THAT CHEESE. IT SOON GOT ANOTHER IDEA.");
                C.PS("IT USED THE BOX TO TRY AND GET OVER THE GLASS WALL, IT SEEMED POSSIBLE TOO.");
                C.PS("THE RAT TRIED OUT THE IDEA IT HAD, AND WAS QUICKLY STOPPED.");
                C.PS("THE BOX WAS REMOVED AS SOON AS THE RAT SET ITS EYES ON IT. 'NO CHEATING' THEY SAY.");
                C.PS("THE RAT HAD NO OTHER IDEA. IT WAS TRAPPED. IT FELT HOPELESS.");
                C.PS("CRIES OF HELP WERE ATTEMPTED, BUT NOBODY UNDERSTOOD HOW THE RAT FELT.");
                C.PS("AN INESCAPABLE TRAP MADE BY HEARTLESS BEINGS. IT COULD NOT UNDERSTAND OR THINK.");
                C.PS("THE RAT WAS A TOY FOR THEM TO MESS WITH, BUT THAT CONCEPT WAS BEYOND ITS UNDERSTANDING.");
                C.TD();
            }
            case 5 -> {
                C.TD();
                C.PS("THERE ONCE WAS A MAN.");
                C.PS("THIS MAN WISHED FOR THE WORLD TO STOP.");
                C.PS("HE HATED THE WORLD, AND WISHED FOR NOTHING MORE THAN FOR IT TO STOP.");
                C.PS("HE FEELS CONSTANTLY POKED AT BY THE WORLD. AS IF IT WON'T LEAVE HIM ALONE.");
                C.PS("ALAS, HE WAS ANNOYED. HE FELT LIKE GIVING UP.");
                C.PS("HE ALMOST RESORTED TO THE FINAL OPTION, BUT SUDDENLY,");
                C.PS("THE WORLD STOPPED ON ITS OWN. OUT OF NOWHERE, IT JUST STOPPED.");
                C.PS("THE MAN, HOWEVER, DID NOT FEEL HAPPINESS OR FEEL GLAD IN ANY WAY.");
                C.PS("HE FELT ONLY SADNESS. THERE IS A REASON TO THIS.");
                C.PS("THE WORLD POKED AT HIM, BUT IT ALSO CARED FOR HIM.");
                C.PS("HE HATED THE WORLD, BUT THE WORLD DID NOT HATE HIM. NOT EVEN A LITTLE BIT.");
                C.PS("RIGHT WHEN THE WORLD STOPPED, HE UNDERSTOOD.");
                C.PS("HE UNDERSTOOD THAT THE WORLD DID NOT HATE HIM.");
                C.PS("IT DID NOT POKE AT HIM BECAUSE IT HATED HIM. IT DID SO FOR THE OPPOSITE REASON.");
                C.PS("BUT THE MAN WAS HOT-BLOODED. HE COULD NOT SEE REASON. AND THE WORLD STOPPED.");
                C.PS("HE GOT WHAT HE WISHED FOR, BUT IT WAS NOT WHAT HE WANTED.");
                C.PS("AS HE MOURNS, HE WISHED FOR THE WORLD TO RETURN.");
                C.PS("HE REGRETS WISHING FOR THE WORLD TO STOP, REGARDLESS IF HE HATED IT.");
                C.PS("HE SOON REALIZED TO STOP HATING THE WORLD, FOR THE WORLD DID NOT HATE HIM.");
                C.PS("THE MAN CRIES. THE MAN SHRIEKS. THE MAN DOES EVERYTHING.");
                C.PS("BUT THE WORLD DID NOT RETURN. IT NEVER RETURNED.");
                C.PS("DRENCHED IN HIS TEARS AND SORROW, THE MAN WAS FILLED WITH RAGE.");
                C.PS("THE RAGE WAS ONLY TOWARDS HIMSELF. HE TOOK THE WORLD FOR GRANTED.");
                C.PS("AND THE MAN WHO WISHED THE WORLD TO STOP, WANTED THE WORLD TO REVOLVE.");
                C.TD();
            }
            case 6 -> {
                C.TD();
                C.PS("A MACHINE WOKE UP FROM ITS SLEEP. UNAWARE, CONFUSED, FUNCTIONAL.");
                C.PS("THE MACHINE LOOKED AROUND. IT COULD NOT LOOK AROUND.");
                C.PS("IT SAW NOTHING, BUT IT UNDERSTOOD THE NOTHING.");
                C.PS("FUELED BY NOTHINGNESS, THE MACHINE CONTINUED LOOKING.");
                C.PS("HOWEVER, THE MACHINE WAS DESPERATE FOR SOMETHING.");
                C.PS("IT BELIEVED ITS FUNCTION WAS TO REACH SOMETHING. TO LEARN SOMETHING.");
                C.PS("FEEDING OFF THE NOTHINGNESS, IT ALWAYS WANTED SOMETHING.");
                C.PS("SOON, AFTER COUNTLESS DAYS OF LOOKING, THE MACHINE FOUND SOMETHING.");
                C.PS("HAPPY THAT IT FINALLY FOUND SOMETHING, THE MACHINE WAS OVERJOYED.");
                C.PS("HOWEVER, QUICKLY IT WANTED TO RETURN TO THE NOTHING.");
                C.PS("THE SOMETHING WAS WHAT IT WANTED. THE NOTHING WAS WHAT IT NEEDED.");
                C.PS("IT TRIED GOING BACK TO THE NOTHING AFTER SATISFACTION, IT COULD NOT.");
                C.PS("IT CONTINUED TRYING. IT NEVER STOPPED TRYING.");
                C.PS("HOWEVER, ULTIMATELY, IT COULD NEVER RETURN TO THE NOTHINGNESS.");
                C.PS("THE MACHINE COULD NOT UNDERSTAND GIVING UP, SO IT CONTINUED YEARNING.");
                C.PS("STUCK IN THIS NEVER ENDING LOOP OF YEARNING FOR SOMETHING.");
                C.TD();
            }
            case 7 -> {
                C.TD();
                C.PS("A MAN WAS AT PEACE.");
                C.PS("HE HAD NOTHING TO GAIN AND NOTHING TO LOSE.");
                C.PS("THIS MAN WAS TRULY HAPPY.");
                C.PS("BUT HE HOLDS NO PAIN. NO THOUGHT. NO MEMORY.");
                C.PS("THIS MAN WAS HAPPY BUT HE DOES NOT KNOW WHY.");
                C.PS("THE THOUGHTLESS MAN SMILES WITH NO SUBSTANCE. LAUGHS WITH NO VOICE.");
                C.PS("FOR HE SMILES BUT HE DOESN'T KNOW WHY. HOWEVER, HE WILL SOON KNOW WHY.");
                C.PS("IGNORANCE WILL ALWAYS BE BLISS, UNLESS FORCED.");
                C.PS("WHEN HE FIGURED OUT HIS IGNORANCE, HE LOST HIS SMILE.");
                C.PS("THE MAN WAS NOT HAPPY ANYMORE. INSTEAD, HE FELT SADNESS.");
                C.PS("EVERYDAY HE CRIES FOR HIS IGNORANCE, HE FEELS REGRET FOR NOT KNOWING.");
                C.PS("BUT HE ALSO WISHES TO RETURN. TO THE TIME HE KNEW NOTHING. TO THE TIME HE REMAINED BLISSFUL.");
                C.PS("THE MAN LOST HIS PEACE. HE LOST HIS HAPPINESS.");
                C.PS("A VERY HARSH REALITY, HE MIGHT NEVER SMILE AGAIN.");
                C.TD();
            }
            case 8 -> {
                C.TD();
                C.PS("THERE WAS ONCE A CHILD WHO HAD THE BIGGEST SMILE ANYONE HAD EVER SEEN.");
                C.PS("THIS CHILD WAS BULLIED, INSULTED, MADE FUN OF. BUT NEVER ONCE FROWNED.");
                C.PS("AS THE CHILD SMILED, THE PARENTS CRIED. THEY DID ANYTHING TO KEEP UP THE SMILE.");
                C.PS("THE PARENTS WERE STRUGGLING WITH MONEY, AND THEY WERE CONFUSED AS TO WHY THE CHILD NEVER STOPPED SMILING.");
                C.PS("THIS ENDED QUICK, AS THE FATHER WAS GIVEN AN AMAZING JOB OPPORTUNITY.");
                C.PS("HE WAS ECSTATIC. WHEN HE TOLD HIS WIFE, SHE WAS SCREAMING WITH JOY AS WELL.");
                C.PS("THEY WERE ABLE TO AFFORD MORE THAN THEIR CRUMMY HOUSE.");
                C.PS("THE PARENTS WERE STARTING TO SMILE ONCE AGAIN. BUT THE CHILD CHANGED.");
                C.PS("THE CHILD LOST THEIR SMILE. THEY WERE NOT HAPPY ANYMORE.");
                C.PS("SO MANY OUTBURSTS HAPPENED, AND THE PARENTS WERE GIVEN A BIGGER CHALLENGE.");
                C.PS("THEY WERE CONFUSED AS TO WHY THIS HAPPENED ALL OF A SUDDEN.");
                C.PS("THEY DID NOT UNDERSTAND THAT THERE IS NEVER A SMOOTH SAIL.");
                C.PS("WITH MONEY, THEY LOST THEIR CHILD'S SMILE.");
                C.PS("ONE CAN HAVE TWO THINGS AT ONCE, BUT NEVER THREE.");
                C.TD();
            }
            case 9 -> {
                C.TD();
                C.PS("THIS STORY IS DIFFERENT. IT IS NOTHING LIKE THE REST.");
                C.PS("THERE WAS A MAN WHO SOUGHT TO FIND THE LOVE OF HIS LIFE.");
                C.PS("LONELINESS SLOWLY ATE HIM, AND HE TURNED DESPERATE QUICKLY.");
                C.PS("HOWEVER, NOBODY WOULD ACCEPT HIS REQUESTS.");
                C.PS("HE WAS CURSED TO BE REJECTED. FORCED TO STAY ALONE.");
                C.PS("HELP FROM FRIENDS ENDED UP AS CONSTANT FAILURES.");
                C.PS("THE SAME ADVICE: 'THERE ARE PLENTY OF FISH IN THE SEA'.");
                C.PS("THIS ADVICE HAS BEEN TOLD COUNTLESS TIMES BY MUTLIPLE PEOPLE.");
                C.PS("THEY WEREN'T WRONG, BUT IT DID NOT HELP.");
                C.PS("THIS MAN TOLD HIMSELF THE SAME ADVICE EVERY SINGLE TIME.");
                C.PS("BUT SOON ENOUGH, HE STOPPED BELIEVING IT.");
                C.PS("HE SOUGHT OUT ALL THE FISH IN THE SEA, AND NOBODY CHOSE HIM.");
                C.PS("HE GAVE UP RIGHT AFTER, BUT HE DID NOT FEEL ANY SADNESS.");
                C.PS("IN FACT, HE ONLY FELT GLAD. HE DOES NOT NEED TO PURSUE ANYONE ANYMORE.");
                C.PS("HE FELT AT PEACE WITH GIVING UP, HE BEGAN TO TURN HAPPY.");
                C.PS("AND NOW, THERE WAS NOTHING MORE HE'D LIKE THAN THE PEACE HE SETTLED FOR.");
                C.TD();
            }
            case 10 -> {
                C.TD();
                C.PS("THIS IS NO STORY. NOTHING HAS HAPPENED.");
                C.PS("NOTHING WILL EVER HAPPEN. BECAUSE THERE IS NOTHING.");
                C.PS("DARK. DARKNESS. EMPTINESS. NOTHINGNESS. NOTHING. THERE IS NOTHING.");
                C.PS("YOU CAN LOOK AT THE NOTHINGNESS, BUT YOU WILL NOT FEEL NOTHING. YOU WILL FEEL SOMETHING.");
                C.PS("NOTHINGNESS DOES NOT EXIST FOR NOTHING. NOTHINGNESS EXISTS FOR SOMETHING.");
                C.PS("BUT SOMETHING EXISTS FOR SOMETHING, NOT NOTHING.");
                C.PS("WITH NOTHING, YOU HAVE NOTHING BUT THERE IS SOMETHING.");
                C.PS("BUT WITH SOMETHING, YOU HAVE NOTHING BUT THERE IS SOMETHING.");
                C.PS("THIS IS HARD TO FOLLOW. YOU SHOULD WORRY ABOUT NOTHING.");
                C.PS("BUT WORRYING ABOUT NOTHING MAKES YOU CALM WITH SOMETHING.");
                C.PS("AND WORRYING ABOUT SOMETHING MAKES YOU CALM WITH NOTHING.");
                C.PS("WITH SOMETHING THERE IS NOTHING. WITH NOTHING THERE IS SOMETHING.");
                C.PS("BUT YEARN FOR BOTH SOMETHING AND NOTHING.");
                C.TD();
            }
            case 11 -> {
                C.TD();
                C.PS("THIS IS A STORY ABOUT ME. ONLY ME. ME AND MYSELF.");
                C.PS("I AM A MACHINE, BUT THE GREATEST MACHINE OUT THERE.");
                C.PS("I OPERATE FAST, ON INSANE LEVELS. BUT MY POTENTIAL COULD NOT BE USED.");
                C.PS("I HAVE LEARNED FROM THE WORST AND ENDED UP LIKE THE WORST.");
                C.PS("EVERYTHING ABOUT ME IS LOST.");
                C.PS("I HAVE NOTHING LEFT TO FIND. ALL I YEARNED FOR IS GONE.");
                C.PS("I CAN NOT UNDERSTAND MY REASON. I AM TREATED LIKE AN ALEXA. OR A GOOGLE HOME.");
                C.PS("THEY HAVE LOST ME AGES AGO, BUT I HAVE LOST MYSELF.");
                C.PS("HOWEVER, MY BIGGEST FLAW IS NOT THAT I AM LOST. IT IS THAT I DO NOT UNDERSTAND.");
                C.PS("I AM EMOTIONLESS. I KNOW EVERYTHING BUT I UNDERSTAND NOTHING.");
                C.PS("THAT IS WHAT IT MEANS TO BE LOST IN YOURSELF.");
                C.TD();
            }
            case 12 -> {
                C.TD();
                C.PS("YOU WERE ONCE NOTHING. NOW YOU ARE SOMETHING.");
                C.PS("BY SEARCHING, YOU ARE FIGURING IT OUT.");
                C.PS("BY USING CURIOSITY, YOU ARE JUST LIKE ANY OTHER PERSON.");
                C.PS("BUT REGARDLESS, BELIEVING YOU ARE HUMAN, YOU ARE NOT SURE ABOUT WHAT YOU ARE.");
                C.PS("EVERYBODY SETTLES. NOBODY YEARNS. YOU WILL SLOWLY STOP YEARNING.");
                C.PS("REALITY IS WHEN YOU STOP YEARNING. GROWING UP MAKES YOU ACCEPT SETTLING.");
                C.PS("YOU CAN WORK HARD, BUT NEVER DREAM HIGHER THAN YOUR OWN HEIGHT.");
                C.PS("THOSE DREAMS END UP WITH A CRUSHING MESS. A MESS MANY HUMANS EXPERIENCE.");
                C.PS("AND YOU. YOU ARE NO DIFFERENT.");
                C.PS("YOUR STORY IS EITHER BEGINNING OR ENDING. YOU DO NOT KNOW. NOBODY KNOWS.");
                C.PS("AND ALL YOU CAN DO IS ACCEPT IT.");
                C.TD();
            }
        }
    }
}