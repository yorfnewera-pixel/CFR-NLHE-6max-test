

Potential-Aware Imperfect-Recall Abstraction with
Earth Mover’s Distance in Imperfect-Information Games
Sam Ganzfried and Tuomas Sandholm
## Computer Science Department
## Carnegie Mellon University
{sganzfri, sandholm}@cs.cmu.edu
## Abstract
There is often a large disparity between the size of a game
we wish to solve and the size of the largest instances solv-
able by the best algorithms; for example, a popular variant
of poker has about10
## 165
nodes in its game tree, while the
currently  best  approximate  equilibrium-finding  algorithms
scale to games with around10
## 12
nodes. In order to approx-
imate equilibrium strategies in these games, the leading ap-
proach is to create a sufficiently small strategic approxima-
tion of the full game, called an abstraction, and to solve that
smaller game instead. The leading abstraction algorithm for
imperfect-information games generates abstractions that have
imperfect  recall  and  are  distribution  aware,  usingk-means
with the earth mover’s distance metric to cluster similar states
together.  A  distribution-aware  abstraction  groups  states  to-
gether at a given round if their full distributions over future
strength are similar (as opposed to, for example, just the ex-
pectation  of  their  strength).  The  leading  algorithm  consid-
ers distributions over future strength at the final round of the
game. However, one might benefit by considering the trajec-
tory  of  distributions  over  strength  in  all  future  rounds,  not
just the final round. An abstraction algorithm that takes all fu-
ture rounds into account is called potential aware. We present
the first algorithm for computing potential-aware imperfect-
recall abstractions using earth mover’s distance. Experiments
on no-limit Texas Hold’em show that our algorithm improves
performance over the previously best approach.
1  Introduction and Background
There is often a large disparity between the size of a game
we wish to solve and the size of the largest instances solv-
able  by  the  best  algorithms.  For  example,  two-player  no-
limit Texas Hold’em poker with common stack sizes (used
in the Annual Computer Poker Competition) has about10
## 165
nodes in its game tree (Johanson 2013), while the currently
best  approximate  equilibrium-finding  algorithms  scale  to
games  with  around10
## 12
nodes  (Zinkevich  et  al.  2007;
Hoda  et  al.  2010).  In  order  to  approximate  equilibrium
strategies in these games, the leading approach is to create a
sufficiently small strategic approximation of the full game,
called anabstraction, and to solve that smaller game instead.
Abstraction in games is quite different than in single-agent
## Copyright
c
©2014, Association for the Advancement of Artificial
Intelligence (www.aaai.org). All rights reserved.
settings. For example, it is not monotonic: if we refine an ab-
straction, we may get strategies that have higher exploitabil-
ity in the original game (Waugh et al. 2009a). Abstraction
has been successfully applied to two-player Texas Hold’em
poker,  first  with  a  manually  generated  abstractions  (Shi
and Littman 2002; Billings et al. 2003), and now with ab-
straction  algorithms  (Gilpin  and  Sandholm  2006;  2007a;
Gilpin, Sandholm, and Sørensen 2008; Waugh et al. 2009b;
Johanson et al. 2013). Many abstraction algorithms work by
coarsening the moves of chance, merging several informa-
tion sets of the original game into single information sets of
the abstracted game. Additional related work on game ab-
straction is discussed in Section 5.
One  approach  for  determining  which  information  sets
should be grouped together is to come up with a measure
of ‘strength’ for each state, then run a clustering algorithm,
such  ask-means,  using  the  difference  in  ‘strength’  as  the
distance metric. For example, in poker, a natural measure of
a hand’s strength is theequity(probability of winning plus
one-half the probability of tying) against a uniform random
draw of private cards for the opponent, assuming a uniform
random rollout of the remaining public (i.e., shared) cards.
This is also known as theexpected hand strength (EHS)met-
ric.  For  example,  if  a  player  is  dealt  two  aces  as  his  two
private cards in Texas Hold’em, he will win 84.93% of the
time and will tie 0.55% of the time against a random hand
assuming  a  random rollout  of  the  public  cards,  giving  his
hand an equity of 0.852. Similarly, the equity of two kings
is 0.824. Since these two values are similar, it is likely that
they would be grouped together by a clustering algorithm.
Early approaches for abstraction in poker used EHS (or EHS
exponentiated to some power) to cluster hands (Billings et
al. 2003; Gilpin and Sandholm 2006; Waugh et al. 2009b;
Zinkevich et al. 2007).
While EHS is a reasonable first-order-approximation for
the strength of a hand, it fails to account for the entire prob-
ability distribution of hand strength. For example, the hands
KcQc (king and queen of clubs) and 6c6d (six of clubs and
six of diamonds) have expected hand strengths of 0.634 and
0.633 respectively, which suggests that they have very sim-
ilar  strength.  However,  looking  at  the  full  distributions  of
hand  strength,  as  opposed  to  just  its  expectation,  paints  a
very different picture, as previous work has shown (Gilpin,
Sandholm, and Sørensen 2007; Johanson et al. 2013). Fig-
Proceedings of the Twenty-Eighth AAAI Conference on Artificial Intelligence
## 682

ures 1 and 2 (which are similar to part of Figure 2 from Jo-
hanson et al.’s work) show the full histograms of expected
hand strength for the two hands, where each bar corresponds
to the probability mass of the given level of hand strength.
For example, if the public board cards are 7dQh4h2s3c, then
KcQc has an equity of 0.856 against a uniform random op-
ponent  hand;  so  the  histogram  entry  corresponding  to  the
column for an equity of 0.84–0.86 is incremented by one for
this hand (prior work has assumed that the equities are di-
vided into 50 regions of size 0.02, as do these figures). As
the figures indicate, despite the fact that these hands have
similar expected hand strengths, their full distributions are
very different. For example, 6c6d frequently has an equity
between 0.5 and 0.7 and rarely has an equity between 0.7
and 0.9, while the reverse is true for KcQc.
An abstraction algorithm that considers the full distribu-
tions of hand strength, as opposed to just the expectation, is
calleddistribution aware. The leading abstraction algorithm
for imperfect-information games generates abstractions that
are distribution aware (Johanson et al. 2013), and it has been
shown empirically that the distribution-aware approach sig-
nificantly  outperforms  EHS-based  approaches  (Gilpin  and
Sandholm  2008;  Johanson  et  al.  2013).  The  natural  met-
ric  for  computing  distances  between  histograms  of  hand-
strength distributions is theearth mover’s distance (EMD).
Informally, EMD is the “minimum cost of turning one pile
into  the  other,  where  the  cost  is  assumed  to  be  amount
of  dirt  moved  times  the  distance  by  which  it  is  moved.”
Earlier work on distribution-aware abstraction used theL
## 2
distance  metric  instead  (Gilpin,  Sandholm,  and  Sørensen
2007),  which  has  been  shown  to  be  significantly  less  ef-
fective  because  it  does  not  properly  account  for  how  far
the “dirt” needs to be moved (only how much needs to be
moved). Using one-dimensional histograms as done above,
EMD can be computed by a straightforward linear time pro-
cedure that scans the histogram and keeps track of how much
dirt needs to be transported between consecutive bins. How-
ever, computing EMD is much more challenging as the di-
mensionality of the data increases, and as we will present
later, multi-dimensional EMD computation will be needed
in more sophisticated abstraction algorithms.
In the domain of Texas Hold’em poker,
## 1
the leading ab-
straction algorithm works as follows (Johanson et al. 2013).
In the first round, there is no card abstraction, and each hand
is in its own bucket. In the second and third rounds, abstrac-
tions are computed as follows. First, an equity histogram is
constructed  for  each  hand,  similarly  to  those  in  Figures  1
and 2. For example, for the flop, we will create a histogram
for the hand where the private cards are Kc3h and the public
cards are KsTd8h. Thenk-means is used to compute an ab-
straction with a desired number of clusters, using the EMD
## 1
Texas Hold’em contains four betting rounds. In the first round
(preflop), players are each dealt two private cards; in the second
round (theflop), three public cards are dealt on the table; in the
third  round  (theturn),  one  more  public  card  is  dealt;  and  in  the
final round (theriver), one final public card is dealt. If play makes
it to the end of the final round, then the player with the best five-
card hand (out of his two private cards and the five public cards)
wins the pot.
between each pair of histograms as the distance metric. One
important feature of these abstractions is that they haveim-
perfect recall: a player can be made to forget information
that  he  knew  earlier  in  the  hand.  For  example,  the  hands
Kc3h-KsTd8h and Kc4h-KsTd8h will likely be grouped to-
gether on the flop, even though the player could distinguish
between Kc3h and Kc4h in the preflop round. Imperfect re-
call abstractions have been demonstrated to lead to signif-
icantly stronger performance than perfect recall for an ab-
straction of a given size, because they allow the player to
have a more refined view of the present since he is allowed
to forget details about the past (Waugh et al. 2009b).
## 2
## That
algorithm computes abstractions for the flop and turn rounds
independently using this approach. It computes the abstrac-
tion for the final round using a different approach (k-means
with L
## 2
over vectors of EHS against first-round clusters of
the opponent).
As  described  above,  the  equity  histograms  for  the  flop
(and turn) rounds consider distributions over future strength
at  the  final  round  of  the  game  (i.e.,  after  all  the  public
cards are dealt). However, it is possible that two flop hands
have very similar distributions over strength after the river
is  dealt,  but  they  realize  the  equity  at  very  different  rates
throughout the hand. Section 2 provides example situations
of how this can arise, both in poker and in a general domain-
independent  game.  Thus,  a  natural  direction  to  explore  is
whether  one  might  benefit  by  considering  the  distribution
over strength in all future rounds, not just the final round.
An  abstraction  algorithm  that  takes  all  future  rounds  into
account is calledpotential aware. Prior work on potential-
aware  abstraction  (Gilpin,  Sandholm,  and  Sørensen  2007)
applied  only  to  perfect-recall  abstraction  and  used  theL
## 2
distance  metric,  both  of  which  have  significant  shortcom-
ings, as described above.
In  this  paper,  we  present  the  first  algorithm  for  com-
puting  potential-aware  imperfect-recall  abstractions,  using
EMD  as  the  distance  metric.  We  design  a  new  abstrac-
tion  algorithm  that  combines  these  three  threads,  each  of
which has been shown helpful separately in the past. Com-
puting  imperfect-recall  abstractions  is  significantly  more
challenging  than  in  the  perfect-recall  case,  since  the  set
of  hands  that  must  be  clustered  at  each  step  is  much
larger. Additionally, computing EMD in this setting is sig-
nificantly  more  challenging  than  in  the  one-dimensional
distribution-aware setting, and is also much more challeng-
ing than computingL
## 2
distance in the potential-aware set-
ting.  The  best  commercially-available  algorithm  for  com-
puting (multi-dimensional) EMD (Pele and Werman 2008;
2009) is far too slow to compute abstractions in poker, and
we develop a fast custom heuristic for approximating EMD
in our setting. Experiments on no-limit Texas Hold’em show
that our algorithm leads to a statistically significant improve-
ment in performance over the previously best abstraction al-
gorithm.
## 2
A downside of using imperfect-recall abstractions is that they
typically cause equilibrium-finding algorithms to lose their conver-
gence guarantees.
## 683

Figure 1: Equity distribution for 6c6d.Figure 2: Equity distribution for KcQc.
2  Potential-Aware Abstraction
In  this  section,  we  present  examples  that  demonstrate  the
difference between potential-aware abstraction and the lead-
ing distribution-aware approach, which considers distribu-
tions  over  future  strength  at  thefinalround  of  the  game.
The examples show that it is possible for two different states
of private information to have very similar (even identical)
histograms over equity at the end of the game, but to real-
ize this equity in very different ways throughout the play of
the game. We first present a domain-independent example
in Section 2.1, followed by an example of a poker situation
demonstrating this phenomenon in Section 2.2.
2.1  Domain-Independent Example
We consider the following game. A player is initially given
private signalx
i
, and then chance makes up to two moves
before the game ends. The information equity trees forx
## 1
andx
## 2
are given in Figures 3 and 4. The opponent is also
given a private signal from some distribution, and the equi-
ties of the initial player against the opponent’s distribution
are  given  in  the  leaves  of  the  trees.  If  the  player  hasx
## 1
## ,
then  chance  selects  the  right  branch  with  probability  1  in
the first round, then selects each branch with probability
## 1
## 2
in the second round. If chance selects the right branch in the
second round, then the player has an equity of 1; otherwise,
he has equity 0. If the player hasx
## 2
, then chance selects each
branch with probability
## 1
## 2
in the first round, and selects the
right branch with probability 1 in the second round (for each
choice of actions in the first round). If chance selected the
left branch in the first round, then the player’s equity is 0; if
chance selected the right branch in the first round, then his
equity is 1.
If  we  use  the  traditional  distribution-aware  approach  of
considering equity assuming the end of the game is reached,
then equity histograms for bothx
## 1
andx
## 2
are the same and
given in Figure 5: with probability
## 1
## 2
, the player will have
equity 0, and with probability
## 1
## 2
, he will have equity 1. For
this  example,  we  assume  that  the  equities  are  broken  into
five equally-sized intervals. (Several of the strongest poker
agents  use  fifty  intervals  each  of  width  0.02.)  Since  these
histograms are identical, the EMD between the two states
corresponding to the two private signals respectively would
be zero, so they would be treated as being identical.
However, these two states are actually quite different if
we consider how the equity changes between the first and
second round, as opposed to just jumping to the end of the
game.  Withx
## 2
,  the  player  will  know  for  sure  whether  he
has an equity of 0 or 1 after chance’s first move, while with
x
## 1
he will not. To perform potential-aware abstraction, the
first step is to compute the histograms for both players at
the possible states in the second round. The histogram for
x
## 1
after chance selects the right branch (i.e., at B) is also
the histogram given in Figure 5; the histogram forx
## 2
after
chance selects the left branch (i.e., at D) has unit mass in the
left-most column (equity of 0–0.2); and the histogram forx
## 2
after chance selects the right branch (i.e., at E) has unit mass
in the right-most column (equity of 0.8–1).
Next,  we  compute  the  histograms  forx
## 1
andx
## 2
at  the
first round, with respect to the possible states that could be
reached at the second round. The histogram forx
## 2
is given
in  Figure  6.  The  possible  statesB,D,  andEcorrespond
to the second round states in the information equity trees.
We  omit  additional  states  that  could  have  originated  from
x
## 3
## ,x
## 4
, etc. (they will all have probability 0). As the figure
shows, withx
## 2
the player will be in statesDandEwith
probability
## 1
## 2
.The histogram forx
## 1
will have unit mass in
the column for stateB. Unlike the histograms above, whose
x-axis is cardinal (i.e., equity), the x-axis of these histograms
is not even ordinal (the next-round states can be listed in any
arbitrary order).
To  transform  this  new  histogram  forx
## 1
into  the  his-
togram  forx
## 2
,  we  must  move  a  mass  of
## 1
## 2
from  the  B
column  to  both  the  D  and  E  columns.  Thus,  the  EMD  is
## 1
## 2
d(B, D)+
## 1
## 2
d(B, E), where theground distancesd(B, D)
andd(B, E)are  computed  using  the  second-round  his-
tograms described above. To transform the histogram at B
into the histogram at D, we must move a mass of
## 1
## 2
from
the rightmost column to the leftmost column; sod(B, D) =
## 1
## 2
·4 = 2. Similarly,d(B, E)also equals 2. So the EMD
between the two, non-ordinal first-round histograms is
## 1
## 2
## ·
## 2 +
## 1
## 2
·2 = 2. Thus, potential-aware abstraction will treat
x
## 1
andx
## 2
differently, and potentially group them into differ-
ent clusters (while the distribution-aware approach will treat
them as identical, as shown above).
## 684

Figure 3: Information equity tree for private signalx
## 1
## .
Figure 4: Information equity tree for private signalx
## 2
## .
Figure 5: Histogram of equity for both private information
x
## 1
andx
## 2
at round 1, assuming the game reaches the end
of the final round.
Figure 6: Histogram for private signalx
## 2
at round 1 over
non-ordinal information states at round 2.
## 2.2  Poker Example
In Section 1 we provided a canonical example of two Texas
Hold’em  hands  with  similar  EHS,  but  very  different  his-
tograms over equity at the end of the hand (KcQc vs. 6c6d).
We now present an example of two hands that have simi-
lar histograms over equity at the final round (and thus also
similar EHS), but realize their equity in very different ways
throughout the hand.
Consider  the  two  flop  hands  TcQd-7h9hQh  and  5c9d-
3d5d7d  (the  first  two  cards  are  the  private  cards,  and  the
next three are the public flop cards). These are both relatively
strong hands. The first hand has top pair (a pair of queens),
and the second hand has second pair (a pair of fives) plus a
flush draw (another diamond on the board would complete
a flush). The hands both have very similar EHS, assuming
both  the  turn  and  river  are  dealt;  TcQd-7h9hQh  has  EHS
0.683  and  5c9d-3d5d7d  has  EHS  0.679.  These  two  hands
also  have  very  similar  full distributions over  equity at  the
final round, as can be seen in Figures 7 and 8. The EMD
between these two distributions is 0.559, where the unit is
equity intervals (assuming fifty intervals of width 0.02, and
a total unit mass for both distributions). This value is suffi-
ciently low that these two hands are grouped into the same
bucket  by  the  leading  distribution-aware  abstraction  algo-
rithm (and therefore, they would be treated as being identical
by equilibrium-finding algorithms).
However, despite similarities between the equity distribu-
tions after the river is dealt, these two hands realize their eq-
uity very differently throughout the hand. Figures 9 and 10
show their expected hand strength distributions on the turn
(the next public card), assuming a uniform random distribu-
tion for the river card and the opponent’s cards. These two
distributions are very different; for example, a large portion
of the time TcQd-7h9hQh will have a turn equity between
0.68 and 0.78, while 5c9d-3d5d7d has a large portion of its
equity in the 0.56–0.66 range.
We note that Figures 9 and 10 depict the distributions of
expected turn hand strength, as opposed to full distributions
over distributions of river hand strength (since each turn card
will lead to a distribution over strength in the next round, not
a single value). For example, for the hand TcQd-7h9hQh, if
the turn card is Ad, the hand’s expected equity, assuming a
uniform random river card and uniform random hand for the
opponent, is 0.695 (though it will vary for individual river
cards); so the interval for 0.68–0.7 in the histogram would
be  incremented  by  one  for  that  turn  card.  This  is  signifi-
cantly  more  simplistic  than  our  potential-aware  approach,
which takes into account the full distribution of turn ‘buck-
ets’, which are themselves distributions over equity intervals
after the river. However, comparing these distributions is still
useful for several reasons. First, if the full distributions over
turn hand strength (which are, themselves, distributions over
river hand strength) were similar, then the distributions over
the  expectation  of  turn  hand  strength  distributions  would
necessarily be similar as well; thus, the fact that the expec-
tation distributions differ significantly indicates that the full
distributions also differ significantly. And second, it is not
## 685

Figure  7:  Equity  distribution  for  TcQd-7h9hQh  on  the
river (final betting round).
Figure 8: Equity distribution for 5c9d-3d5d7d on the river
(final betting round).
Figure  9:  Equity  distribution  for  TcQd-7h9hQh  on  the
turn (next betting round). Each point is the expected hand
strength for a given turn card assuming a uniform random
distribution for the river card and the opponent’s cards.
Figure  10:  Equity  distribution  for  5c9d-3d5d7d  on  the
turn (next betting round). Each point is the expected hand
strength for a given turn card assuming a uniform random
distribution for the river card and the opponent’s cards.
feasible  to  compactly  represent  the  full  distributions  visu-
ally, while the distributions of expected hand strength can
be represented easily as two-dimensional histograms.
While  the  EMD  between  the  river  equity  distributions
depicted  in  Figures  7  and  8  is  0.559,  the  EMD  between
full turn distributions using the potential-aware approach is
4.519 (using comparable units). Potential-aware abstraction
is able to correctly identify that these hands are quite differ-
ent, and places them into different buckets due to their large
EMD (while the prior abstraction algorithm places them in
the same bucket).
3  Algorithm for Potential-Aware
Imperfect-Recall Abstraction, with EMD
In this section, we present our new algorithm for computing
potential-aware  imperfect-recall  abstractions  using  EMD.
We first present our main algorithm, followed by a heuris-
tic for quickly approximating the EMD in our setting that
we use to make the algorithm practical for large games such
as Texas Hold’em.
Our abstraction algorithm, depicted in Algorithm 1, works
as follows. Assume the information tree hasr+ 1levels (0–
r), and for each leveln, a number of clustersC
n
is specified
as input. For the final roundsn= ˆr, . . . , r, an arbitrary ab-
straction algorithmS
n
is used, with distance functiond
n
, to
produceC
n
clusters; letA
n
denote the resulting abstraction,
and letm
n
i
denote the mean of thei’th cluster inA
n
## . Next,
we compute the abstraction at roundˆr−1as follows. First,
we compute the distanced
n
i,j
between each pair of round-
(n+1)  meansm
n+1
i
andm
n+1
j
,  using  the  distance  metric
d
n+1
. Next, we compute histogramsH
n
## (x
n
), where thei-th
element ofH
n
## (x
n
)is the fraction of the time that chance’s
next move will sendx
n
into clusteriinA
n+1
. Finally, we
compute  the  abstractionA
n
at  roundn,  by  clustering  the
histogramsH
n
intoC
n
clusters using clustering algorithm
## L
n
(prior work in poker usesk-means). The distance met-
ric used, denotedd
n
, is the EMD between the histograms,
usingd
n
i,j
as the ground distance between componentsiand
jof a histogram. We then compute the new cluster means
m
n
i
, and continue in the same fashion forn= ˆr−2, . . . ,0.
The resulting abstraction,A
n
, has imperfect recall since we
cluster all of the histogramsH
n
## (x
n
)without any regard for
the information known at the corresponding states at previ-
ous stages of the game, and potentially we cluster two states
together that contain information that we could distinguish
between at earlier rounds.
To  compute  the  distances  in  the  main  loop  of  Algo-
rithm 1 we implemented the fastest commercially-available
## 686

Algorithm 1Main algorithm for computing potential-aware
imperfect-recall abstractions
Inputs:{C
n
}:n= 0, . . . , r;{S
n
## },{d
n
}:n= ˆr, . . . , r;
## {L
n
## }:n= 0, . . . ,ˆr−1
forn=rtoˆrdo
Compute  abstractionA
n
withC
n
clusters  using  ab-
straction algorithmS
n
with distance functiond
n
end for
forn= ˆr−1to0do
fori= 1toC
n
do
m
n+1
i
←mean of clusteriinA
n+1
end for
fori= 1toC
n
## −1do
forj=i+ 1toC
n
do
d
n
i,j
←distance betweenm
n+1
i
andm
n+1
j
using
distance functiond
n+1
end for
end for
foreach pointx
n
at roundndo
## H
n
## (x
n
)←histogram  forx
n
over  clusters  from
## A
n+1
end for
Compute  abstractionA
n
over  histogramsH
n
using
clustering algorithmL
n
and distance functiond
n
## (i.e.,
EMD withd
n
i,j
as the ground distance) to produceC
n
clusters
end for
multi-dimensional EMD algorithm (Pele and Werman 2008;
2009); however, it was far too slow for the domain of Texas
Hold’em. So we were forced to develop a faster heuristic for
approximating EMD in this setting. Our heuristic is given in
Algorithm 2. The context is that we are runningk-means to
compute an abstraction at levelnof the tree, for which we
must compute the distance between each ‘point’ and each
mean.  The  ‘points’  correspond  to  the  histogramsH
n
## (x
n
## )
over clusters inA
n+1
, and were computed in the previous
step  of  Algorithm  1.  The  na
## ̈
ıve  way  of  representing  them
would be as vectors of dimensionC
n+1
. However, this vec-
tor may be very sparse. For example, ifC
n+1
## = 5000(as
in our experiments), but the current point can only transi-
tion  into  50  next-round  clusters  with  positive  probability,
we would like to take advantage of a sparser representation
rather than represent it as a vector of size 5000. Instead, we
represent the point as a vector of length 50 (in this example),
where each index corresponds to the index of the next-round
cluster we transition to. For example, if a point can transi-
tion to clusters 3, 5, or 10, for different chance moves, then
we represent the point as the vector(3,5,10),where each of
these will have probability
## 1
## 3
. Each pointx
n
in Algorithm 2
corresponds to such a vector, whereNdenotes the length.
For simplicity we assume that all of the elements are dis-
tinct, though repeated elements can be dealt with straight-
forwardly.
We  similarly  take  advantage  of  sparsity  in  representing
the means. While each mean could potentially haveC
n+1
entries, many of these entries may be zero. Instead, we sim-
Algorithm 2Algorithm for efficiently approximating EMD
in our setting
## Inputs:  Pointx
n
withNelements;  meanmwithQele-
ments; sortedDistances[i][j], orderedClusters[i][j], for1≤
i≤C
n+1
,1≤j≤Q
targets[]←array of sizeNwith all elements equal to
## 1
## N
meanRemaining[]←copy ofm
done[]←array of sizeNwith all elements set to false
totCost←0
fori= 1toQdo
forj= 1toNdo
ifdone[j] ==true then
continue
end if
pointCluster←x
n
## [j]
meanCluster←orderedClusters[pointCluster][i]
amtRemaining←meanRemaining[meanCluster]
ifamtRemaining == 0then
continue
end if
d←sortedDistances[pointCluster][i]
ifamtRemaining<targets[j]then
totCost += amtRemaining * d
targets[j] -= amtRemaining
meanRemaining[meanCluster]←0
else
totCost += targets[j] * d
targets[j]←0
meanRemaining[meanCluster] -= targets[j]
done[j]←true
end if
end for
end for
returntotCost
ply represent the meanmas the vector of the nonzero en-
tries,  of  which  we  assume  there  areQ.  In  order  to  iden-
tify which clusters the entries correspond to, and to make
our  overall  implementation  more  efficient,  we  utilize  sev-
eral  data  structures.  First,  we  precompute  an  array  called
sortedDistances, where sortedDistances[i][j] is the distance
between next-round clusteriand thej-th closest cluster toi
for which the current mean has non-zero probability, where
distances have already been computed usingd
n+1
. We also
use a structure orderedClusters, where orderedClusters[i][j]
is  the  index  of  the  cluster  that  the  mean  assigns  non-zero
probability to that isj-th closest to clusteri. These arrays
are precomputed in advance of the EMD computation; while
they require some time to compute, the EMD computations
are by far the bottleneck of the algorithm. This additional
computation helps us overall since it significantly speeds up
the EMD computations.
Given these data structures as input, we now approximate
EMD as follows. First, we start with the first entry ofx
n
; we
call  the  round-(n+1)  cluster  to  which  this  belongs  ‘point-
Cluster.’ We then find the closest cluster to pointCluster that
corresponds to a nonzero element in the mean; this will be
## 687

the element orderedClusters[pointCluster][1], which we call
‘meanCluster.’ We shift as much mass as possible between
from this mean element to the corresponding point element.
The cost is increased by the amount of mass we shift mul-
tiplied by the distance. We then update the remaining point
mass and mean mass at the elements we have considered,
and continue for the remaining point indicesj= 2, . . . , N.
Next,  we  seti= 2,  and  repeat  the  same  procedure,  now
shifting as much mass as is available from the second clos-
est  nonzero  mean  cluster  to  each  cluster  of  the  point.  We
repeat this fori= 3, . . . , Q, until the mean vector has been
fully transformed into the point vector. We then output the
resulting total cost.
As mentioned above, the fastest commercially-available
algorithm  for  computing  EMD  (Pele  and  Werman  2008;
2009) is far too slow to be effective in Texas Hold’em. (This
was despite the fact that we integrated the data structures de-
scribed above with this algorithm to exploit sparsity, as well
as  applied  several  other  enhancements  to  improve  perfor-
mance ofk-means, such as a pruning technique that exploits
the triangle inequality (Elkan 2012) and parallelizing each
step  using  64  cores.)  For  the  point-mean  distances  in  the
first round ofk-means, the exact EMD algorithm averaged
11.4 ms per computation, while our heuristic averaged only
0.008 ms per computation. Furthermore, the exact algorithm
scales extremely poorly as the dimensionality of the inputs
increases. Since the initial means are themselves data points,
their dimensionality is small; however, for future rounds of
k-means, the means are weighted averages over all points in
their cluster, and have higher dimensionality. Our new algo-
rithm performs well even in the future rounds ofk-means
as this dimensionality increases, while the exact algorithm
scales very poorly.
There are many potential further improvements to approx-
imating EMD and doing clustering in this context. However,
even with the techniques we already developed and tested,
the approach outperforms the previously best abstraction al-
gorithm, as the experiments in the next section will show.
## 4  Experiments
We evaluated our abstraction algorithm in a very large se-
quential  imperfect-information  game,  two-player  no-limit
Texas Hold’em. While our abstraction algorithm works for
any  number  of  agents  and  does  not  assume  a  zero-sum
game, we focused on this two-player zero-sum game in the
experiments—as is most common in this field—so that we
can compute a near equilibrium to a large abstraction, and
thereby evaluate the results.
4.1  Head-to-Head Performance vs. Best Prior
## Abstraction Algorithm
We ran two different sets of experiments corresponding to
two  different  manually-generated  betting  abstractions.  In
both experiments, we compared performance to the previ-
ously best abstraction algorithm (Johanson et al. 2013). In
each experiment, we used the same betting abstraction for us
and for the benchmark. In the first experiment, we used the
betting abstraction that was used by the agent that finished
in 2nd place in the 2012 Annual Computer Poker Competi-
tion. We chose to test on this relatively small betting abstrac-
tion so that we could obtain a good level of convergence to
equilibrium in the abstract game. In the second experiment,
we used a very large betting abstraction; it is about 8 times
larger than the version used by our 2013 competition agent,
and currently beats the winner from the 2013 competition.
In both experiments, we used 169, 5000, 5000, and 5000
card buckets respectively in the four betting rounds for both
the new algorithm and the prior algorithm. This card abstrac-
tion is used by our strongest agent that now beats the win-
ner  from  the  2013  competition,  and  was  also  used  by  the
2013 competition agent that finished in 3rd. Also, as is typ-
ical nowadays among all the top teams, the first round has
169 buckets corresponding to no abstraction at all.
In each of the two experiments, we created an agent that
was identical to the corresponding opponent, except that it
used our new algorithm to compute the abstraction for the
flop  round  (i.e.,  second  betting  round).  For  both  flop  ab-
straction algorithms, we conducted 25 restarts using thek-
means++ initialization procedure (Arthur and Vassilvitskii
2007), and selected the run that produced the lowest within-
cluster sum of squares. We chose to focus on the flop round
for the following reasons. First, the strongest agents do not
use any abstraction preflop (i.e., on the first betting round),
and there is no potential on the river (i.e., last betting round)
since no further cards will be dealt; so potential-aware ab-
straction would not help on those rounds. The approach is
potentially  useful  on  the  turn  (i.e.,  third  betting  round)  as
well, but it appears to be computationally intractable, even
using our fast heuristic for EMD (there are around 1.3 mil-
lion hands to be clustered on the flop, and 55 million on the
turn). For each of the generated abstractions (two new and
two benchmarks), we computed an approximate equilibrium
for the abstraction using a sampled version of counterfactual
regret minimization (Lanctot et al. 2009).
In each of the two experiments, we ran 20,000 duplicate
matches between our new agent and the respective bench-
mark agent. In both experiments, our new approach led to a
statistically significant improvement over the old approach.
In the first experiment, the new agent beat its benchmark by
2.58  milli big blinds per hand (mbb/h) with a  95%  confi-
dence interval of±1.56mbb/h. In the second experiment,
the  new  agent  beat  its  benchmark  by  2.22  mbb/h  (±1.28
mbb/h).
4.2  Evaluating the Approximation of
Potential-Aware EMD
To  evaluate  how  closely  the  distance  computed  by  Algo-
rithm 2 approximates the true potential-aware EMD, we re-
peatedly  generated  the  histograms  (over  turn  buckets)  for
two random flop hands, and computed the exact EMD be-
tween  the  histograms.  If  this  distance  was  less  than  some
threshold, then we also ran Algorithm 2 to approximate the
EMD. (We chose a threshold of 3000
## 3
since it appears that
the vast majority of the closest point-mean distances were
## 3
These values must be divided by 1081 for the total histogram
mass to be normalized to one.
## 688

in the 0–2500 range, and we are mostly interested in how
well our heuristic does at approximating EMD for the point-
mean pairs with low distance, since those represent the clus-
ter to which a point might actually be assigned. We are not
as concerned about how far off our heuristic is for distances
that are extremely large, as they will likely be pruned and
have no chance of being assigned as the closest mean.) We
computed the relative error between the EMD computed by
our heuristic and the true EMD, and averaged it over many
samples. Our algorithm had average relative error of 0.1496
(with 95% confidence interval±0.0014).
For comparison, we also computed the EMD between the
same flop hands using the previously best distribution-aware
approach,  where  the  histograms  consider  equity  assuming
the  end  of  the  game  is  reached.  That  approach  produced
an average relative error of 0.2084 (±0.0014) compared to
the potential-aware EMD over the same sample. Thus, our
potential-aware, but heuristically-computed EMD, approxi-
mates the true potential-aware EMD 28.2% better than the
prior approach for computing EMD, which used exact cal-
culation but was not potential aware.
Though we already demonstrated the superiority of our
abstraction  algorithm  over  the  prior  algorithm  in  the  ex-
periments described in Section 4.1, these results provide a
further  sanity  check  that  Algorithm  2  does  in  fact  lead  to
a  better  degree  of  approximation  of  potential-aware  EMD
than the prior method. The results also indicate that there is
still room for significant improvement toward more accurate
potential-aware EMD computation.
## 5  Additional Related Research
The  closely  related  research  on  game  abstraction  was  al-
ready  cited  earlier.  Here  we  discuss  additional  papers  on
game abstraction.
The GameShrink algorithm computes a lossless informa-
tion abstraction (Gilpin and Sandholm 2007b) in a class of
games, but in very large games like Texas Hold’em, lossy
abstraction is needed to create an abstract game that is small
enough to solve to near equilibrium.
This  paper,  like  most  prior  work  on  game  abstraction,
has been about information abstraction. Typically action ab-
straction (e.g., selection of bet sizes to include in the abstrac-
tion in poker) is done manually—as was done in this paper
as well. There is also an emerging thread of research into
automated action abstraction (Hawkin, Holte, and Szafron
2011; 2012; Brown and Sandholm 2014).
Recently, a framework and algorithms were presented for
lossy abstraction with bounds on the resulting equilibrium
quality. It applies both to information and action abstraction.
This  is  for  stochastic  games  (Sandholm  and  Singh  2012),
and more recently extensive-form games (Kroer and Sand-
holm 2014). For extensive-form games, the algorithm—an
integer program—only scales to a tiny poker game (with a
five-card  deck  and  two  rounds  of  betting  with  a  fixed  bet
size), so the algorithm is not applicable to large poker vari-
ants,  such  as  Texas  Hold’em.  That  framework  does  apply
to Texas Hold’em, and it remains an open question whether
scalable algorithms within that framework can be developed.
Other recent work provides a regret bound in the full game
for a class of games for strategies produced by applying an
equilibrium-finding  algorithm  (counterfactual  regret  mini-
mization)  to  abstractions  (Lanctot  et  al.  2012).  The  regret
bound applies for both perfect and imperfect recall abstrac-
tions of games from the given class.
6  Conclusions and Future Research
We  presented  the  first  algorithm  for  computing  potential-
aware imperfect-recall abstractions using earth mover’s dis-
tance  as  a  distance  metric.  This  is  the  first  algorithm  that
combines potential-aware abstraction with imperfect recall.
It is also the first time earth mover’s distance has been used
in  potential-aware  abstraction.  Both  of  these  are  concep-
tually clear improvements, and experiments showed in the
large that the new algorithm outperforms the best prior ab-
straction algorithm with statistical significance.
A  future  direction  would  be  to  develop  more  accurate
and/or faster heuristics for approximating EMD in our set-
ting,  or  faster  algorithms  for  computing  exact  EMD.  One
technique for achieving the latter would be to use a cluster-
ing algorithm that selects cluster centers that have lower di-
mensionality than the centers selected byk-means, since the
best commercially-available algorithm for computing EMD
is  slow  for  points  with  high  dimensionality.  A  promising
approach is thek-medoids algorithm, which only uses data
points as the cluster centers. However,k-medoids is signifi-
cantly slower thank-means and it requires additional mem-
ory, so it is unclear whether it will be feasible to apply to
poker or other large games. We would also like to extend
our approach to the turn round as well, though this appears
to be infeasible, even using our fast heuristic, due to the large
number of turn hands that need to be clustered. One tech-
nique that may help is to sample a subset of the turn hands
and perform clustering only over that subset.
We  expect  our  algorithm  to  be  applicable  to  games  be-
yond no-limit Texas Hold’em (NLHE). We would expect it
to  lead  to  an  even  more  significant  performance  improve-
ment over the prior approach in certain other popular vari-
ants of poker, such as pot-limit Omaha Hold’em (aka PLO),
since the strength of hands changes much more significantly
between rounds than in Texas Hold’em, so taking the full
trajectory of strength into account would be more important
in that game. PLO also has a significantly larger state space
than Texas Hold’em (in all rounds), and therefore we would
expect abstraction to play a larger role than in NLHE. In par-
ticular, there are many more hands for the preflop and flop
rounds in PLO than in NLHE, and we expect our approach
to help the most when performing abstraction in the earlier
rounds,  since  that  is  where  there  is  the  biggest  difference
between the distribution of hand strength assuming the fi-
nal round is reached and the full trajectory of hand strength
distributions in all future rounds.
We would also like to apply our algorithm to games out-
side of poker. It would be applicable to any large sequential
game of imperfect information where information abstrac-
tion is necessary, and it would be especially useful for games
where the strength of private information can change signif-
icantly between rounds, since the algorithm accounts for the
full trajectory of strength over all future rounds.
## 689

## 7  Acknowledgments
This  material  is  based  on  work  supported  by  the  Nation-
al  Science  Foundation  under  grants  IIS-1320620,  CCF-
1101668, and IIS-0964579, as well as XSEDE computing
resources provided by the Pittsburgh Supercomputing Cen-
ter. We would also like to thank Michael Johanson for pro-
viding us with the data used for Figures 1 and 2.
## References
Arthur, D., and Vassilvitskii, S.  2007.  k-means++: The ad-
vantages of careful seeding.  InProceedings of the Annual
ACM-SIAM Symposium on Discrete Algorithms (SODA).
## Billings, D.; Burch, N.; Davidson, A.; Holte, R.; Schaeffer,
J.; Schauenberg, T.; and Szafron, D.   2003.   Approximat-
ing game-theoretic optimal strategies for full-scale poker. In
Proceedings of the 18th International Joint Conference on
Artificial Intelligence (IJCAI).
Brown,  N.,  and  Sandholm,  T.   2014.   Regret  transfer  and
parameter optimization.   InProceedings of the AAAI Con-
ference on Artificial Intelligence (AAAI).
Elkan, C.  2012.  Using the triangle inequality to accelerate
k-means. InProceedings of the International Conference on
Machine Learning (ICML).
Gilpin, A., and Sandholm, T.   2006.   A competitive Texas
Hold’em poker player via automated abstraction and real-
time equilibrium computation.   InProceedings of the Na-
tional Conference on Artificial Intelligence (AAAI).
Gilpin, A., and Sandholm, T.  2007a.  Better automated ab-
straction techniques for imperfect information games, with
application to Texas Hold’em poker.  InProceedings of the
International Conference on Autonomous Agents and Multi-
Agent Systems (AAMAS).
Gilpin, A., and Sandholm, T. 2007b. Lossless abstraction of
imperfect information games.Journal of the ACM54(5).
Gilpin, A., and Sandholm, T.  2008.  Expectation-based ver-
sus potential-aware automated abstraction in imperfect in-
formation games: An experimental comparison using poker.
InProceedings of the AAAI Conference on Artificial Intelli-
gence (AAAI). Short paper.
Gilpin,  A.;  Sandholm,  T.;  and  Sørensen,  T.  B.2007.
Potential-aware automated abstraction of sequential games,
and holistic equilibrium analysis of Texas Hold’em poker.
InProceedings of the AAAI Conference on Artificial Intelli-
gence (AAAI).
Gilpin,  A.;  Sandholm,  T.;  and  Sørensen,  T.  B.   2008.   A
heads-up no-limit Texas Hold’em poker player: Discretized
betting  models  and  automatically  generated  equilibrium-
finding programs.  InProceedings of the International Con-
ference  on  Autonomous  Agents  and  Multi-Agent  Systems
## (AAMAS).
Hawkin, J.; Holte, R.; and Szafron, D.   2011.   Automated
action abstraction of imperfect information extensive-form
games. InProceedings of the AAAI Conference on Artificial
Intelligence (AAAI).
Hawkin, J.; Holte, R.; and Szafron, D.  2012.  Using sliding
windows to generate action abstractions in extensive-form
games. InProceedings of the AAAI Conference on Artificial
Intelligence (AAAI).
## Hoda,  S.;  Gilpin,  A.;  Pe
## ̃
na,  J.;  and  Sandholm,  T.    2010.
Smoothing techniques for computing Nash equilibria of se-
quential games.Mathematics of Operations Research35(2).
Conference version appeared in WINE-07.
Johanson, M.; Burch, N.; Valenzano, R.; and Bowling, M.
-  Evaluating state-space abstractions in extensive-form
games.  InProceedings of the International Conference on
Autonomous Agents and Multi-Agent Systems (AAMAS).
Johanson, M.   2013.   Measuring the size of large no-limit
poker games. Technical report, University of Alberta.
Kroer, C., and Sandholm, T.   2014.   Extensive-form game
abstraction with bounds.  InProceedings of the ACM Con-
ference on Economics and Computation (EC).
Lanctot, M.; Waugh, K.; Zinkevich, M.; and Bowling, M.
-  Monte Carlo sampling for regret minimization in ex-
tensive games. InProceedings of the Annual Conference on
Neural Information Processing Systems (NIPS).
Lanctot,  M.;  Gibson,  R.;  Burch,  N.;  Zinkevich,  M.;  and
Bowling, M.   2012.   No-regret learning in extensive-form
games with imperfect recall.  InProceedings of the Interna-
tional Conference on Machine Learning (ICML).
Pele, O., and Werman, M.  2008.  A linear time histogram
metric for improved SIFT matching.  InProceedings of the
European Conference on Computer Vision.
Pele,  O.,  and  Werman,  M.    2009.    Fast  and  robust  earth
mover’s distances. InProceedings of the International Con-
ference on Computer Vision.
Sandholm, T., and Singh, S.  2012.  Lossy stochastic game
abstraction with bounds.  InProceedings of the ACM Con-
ference on Electronic Commerce (EC).
Shi,  J.,  and  Littman,  M.   2002.   Abstraction  methods  for
game theoretic poker.  InCG ’00: Revised Papers from the
Second International Conference on Computers and Games.
London, UK: Springer-Verlag.
Waugh,  K.;  Schnizlein,  D.;  Bowling,  M.;  and  Szafron,  D.
2009a.  Abstraction pathologies in extensive games.  InPro-
ceedings  of  the  International  Conference  on  Autonomous
Agents and Multi-Agent Systems (AAMAS).
## Waugh, K.; Zinkevich, M.; Johanson, M.; Kan, M.; Schni-
zlein, D.; and Bowling, M. 2009b. A practical use of imper-
fect recall. InProceedings of the Symposium on Abstraction,
Reformulation and Approximation (SARA).
Zinkevich,  M.;  Bowling,  M.;  Johanson,  M.;  and  Piccione,
C.   2007.   Regret minimization in games with incomplete
information.   InProceedings of the Annual Conference on
Neural Information Processing Systems (NIPS).
## 690