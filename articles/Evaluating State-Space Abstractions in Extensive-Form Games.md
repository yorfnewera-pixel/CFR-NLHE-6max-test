

Evaluating State-Space Abstractions in
Extensive-Form Games
Michael Johanson and Neil Burch and Richard Valenzano and Michael Bowling
University of Alberta
## Edmonton, Alberta
## {johanson,nburch,valenzan,mbowling}@ualberta.ca
## ABSTRACT
Efficient  algorithms  exist  for  finding  optimal  policies  in
extensive-form games.  However, human-scale problems are
typically so large that this computation remains infeasible
with modern computing resources.  State-space abstraction
techniques allow for the derivation of a smaller and strategi-
cally similar abstract domain, in which an optimal strategy
can be computed and then used as a suboptimal strategy in
the real domain. In this paper, we consider the task of evalu-
ating the quality of an abstraction, independent of a specific
abstract strategy.  In particular, we use a recent metric for
abstraction  quality  and  examine  imperfect  recall  abstrac-
tions, in which agents “forget” previously observed informa-
tion to focus the abstraction effort on more recent and rele-
vant state information.  We present experimental results in
the domain of Texas hold’em poker that validate the use of
distribution-aware  abstractions  over  expectation-based  ap-
proaches, demonstrate that the new metric better predicts
tournament performance, and show that abstractions built
using imperfect recall outperform those built using perfect
recall in terms of both exploitability and one-on-one play.
Categories and Subject Descriptors
I.2.1  [Artificial  Intelligence]:   Applications  and  Expert
Systems—Games
## General Terms
## Algorithms
## Keywords
Economic  paradigms::Game  theory  (cooperative  and  non-
cooperative);  Learning and Adaptation::Multiagent Learn-
ing
## 1.  INTRODUCTION
Realistic multiagent settings involve complex, sequential
interactions  between  agents  with  different  perspectives  re-
garding the underlying state of the world.  A general model
for  such  settings  is  the  extensive-form  game  with  imper-
fect information.  While state-of-the-art techniques for ap-
proximating  Nash  equilibria  in  extensive-form  games  [21,
Appears in:Proceedings of the 12th International Confer-
ence on Autonomous Agents and Multiagent Systems (AA-
MAS 2013), Ito, Jonker, Gini, and Shehory (eds.), May,
6–10, 2013, Saint Paul, Minnesota, USA.
## Copyright
c
©2013, International Foundation for Autonomous Agents and
Multiagent Systems (www.ifaamas.org). All rights reserved.
8]  have  made  remarkable  progress  [15,  11],  the  size  of
most real-world settings is beyond the capability of current
solvers.  For example, a common benchmark of progress is
the domain of computer poker.  Current solution techniques
have found approximate equilibria in poker-like games with
as  many  as  88  billion  decision  points  [9],  which  is  still
four  orders  of  magnitude  smaller  than  the  smallest  poker
game played by humans.  The ubiquitous approach to han-
dling  such  human-scale  domains  isabstraction[2,  16,  5],
where  strategically  similar  decision  points  for  the  players
are grouped to construct an abstract game that is tractably
sized  for  current  solution  techniques.   The  solution  of  the
abstract game is then employed in the original game.
While even simple abstraction techniques have been found
to be empirically effective [21], their success is not guaran-
teed.   Waughet al.[18]  gave  surprising  examples  ofab-
straction pathologieswhere strict refinements of abstrac-
tions can result in abstract strategy equilibria that are more
exploitable  in  the  real  game.   While  there  is  little  theory
to guide the construction of abstractions, Gilpin and Sand-
holm [6] presented three methods for empirically comparing
abstraction methodologies:  one-on-one comparison, versus-
equilibrium comparison, and versus-best-response compari-
son.   While  these  remained  the  best-practice  approach  for
abstraction  evaluation,  each  of  these  methods  has  concep-
tual drawbacks:  possible intransitivities, infeasible compu-
tation,  and  not  being  well  correlated  with  actual  perfor-
mance  (respectively).    Johansonet al.[10]  recently  pre-
sented  theCFR-BRalgorithm,  which  computes  the  best
Nash approximation strategy that can be represented in a
given abstraction.  This represents a new, fourth method for
evaluating  abstraction  methodologies:  comparing  the  rep-
resentation power of an abstraction by how well it can ap-
proximate  an  unabstracted  Nash  equilibrium.   In  this  pa-
per,  we will examine the efficacy of this new approach for
evaluating  abstractions,  and  use  it  to  evaluate  several  ab-
straction methodologies in the poker domain.  We show that
not only does it have many desirable conceptual properties
(e.g., transitivity and computational tractability), it is also
empirically well-correlated with the in-game performance of
abstract game equilibria. We demonstrate all of this through
a series of abstraction evaluation experiments.  In particular,
we repeat the Gilpin and Sandholm experiments that con-
cluded that expectation-based abstractions are weaker than
distribution-aware abstractions
## 1
.  We also use this technique
## 1
Gilpin and Sandholm refer to their abstraction technique
as beingpotential-aware, as it is distribution-aware and can
also represent how quickly a hand may change over time.

to  validate  the  efficacy  ofimperfect  recallabstractions,
in which an agent forgets information known in past deci-
sions to refine its representation of its current state.  Such
abstractions are empirically effective [19, 12], but previous
research has not shown a conclusive advantage.  Finally, we
present for the first time the abstraction methodology em-
ployed by Hyperborean,  one of the top competitors in the
## Annual Computer Poker Competition.
## 2.  BACKGROUND
Extensive-form games.Extensive-form games are an in-
tuitive  formalism  for  representing  the  interaction  between
agents and their environment.  These interactions are repre-
sented by a tree, in which nodes represent game states and
edges represent actions taken by one of the agents,i∈N, or
chance,c.  The root of the tree represents the start of the
interaction, and actions are taken until a leaf, i.e.,terminal
nodeis reached.  Each terminal nodez∈Zassigns a utility
to each playeri,u
i
(z).  Inimperfect informationgames,
agents may not be able to observe some of the actions taken
by chance or the other agents.  In the poker setting we use
the termsprivateandpublicto refer to actions visible to
only  one  agent  or  to  all  agents,  although  in  general  other
types of actions are possible.  Each set of game states that
are indistinguishable by the acting agent is called aninfor-
mation set.  When some actions are not observed, an agent
perceives the game not as a tree of game states, but as a tree
of information sets.  Aperfect recallgame has the natural
property that each agent remembers the exact sequence of
its past observations and actions leading to each decision.
Abehavioral strategy(or simply astrategy) for each
playeri,σ
i
,  maps  each  of  playeri’s  information  sets  to  a
probability  distribution  over  the  legal  actions.   Astrat-
egy  profileσis  a  tuple  containing  a  strategy  for  each
player.   Letσ
## −i
refer  to  the  strategies  of  playeri’s  oppo-
nents.  Given a strategy profileσ,  we denote each player’s
expected  utilitybyu
i
(σ).   Given the  opponents’  strate-
giesσ
## −i
, abest responsefor playeriis the strategy that
maximizes  utility  againstσ
## −i
,  whereb
i
## (σ
## −i
)  is  the  util-
ity of the best response strategy when played againstσ
## −i
## .
A  strategy  profileσis  called  an-Nash  equilibriumif
∀i∈N,b
i
## (σ
## −i
## )−u
i
## (σ
i
## ,σ
## −i
)≤.   When=  0,  the  pro-
file is called aNash equilibrium.  In two-player repeated
games where the agents alternate positions, each agent has
one  strategy  for  each  position  and  theirexploitabilityis
their  utility  (averaged  over  all  positions)  against  a  worst-
case adversary who,  in each position,  uses a best-response
to the agent.  In two-player zero-sum games,  a Nash equi-
librium has an exploitability of 0 and thus cannot lose, on
expectation, to any adversary.
Poker is a canonical example of stochastic imperfect infor-
mation extensive-form games.  In this paper we will focus on
two-player limit Texas hold’em, which is one of the variants
played  in  the  Annual  Computer  Poker  Competition.   The
game  begins  with  each  player  being  given  ahandof  two
private cardsthat only they can see or use.  The players’
actions are tobetorcall, placing or matching wagers that
their hand will be the strongest at the end of the game, or
tofoldto concede the game.  This is followed by chance re-
vealing an additional threepublic cardsthat both players
can see and use, and an additional round of betting actions.
After two additional such rounds in which one public card is
revealed and the players act, the game is over and the player
with the strongest hand made of their private cards and the
public cards wins the wagers.  Poker is a repeated game in
which two agents will play a long series of such games with
the overall goal of having the highest total winnings.
Counterfactual Regret Minimization.Counterfactual
Regret Minimization (CFR) is a state-of-the-art algorithm
forsolvingextensive-form  games  (i.e.,  approximating  a
Nash equilibrium strategy) and has been widely used in the
poker domain [21, 9].  Although it is only proven to converge
to a Nash equilibrium in two-player zero-sum perfect recall
games, in practice it appears robust when these constraints
are  violated  as  it  has  been  successfully  applied  to  multi-
player games [14], non-zero-sum games [12], and imperfect
recall games [19].  CFR is an iterative self-play algorithm.
Each player starts with an arbitrary strategy.  On each it-
eration,  the  players  examine  every  decision,  and  for  each
possible action compare the observed value of their current
policy to the value they could have achieved by making that
action instead.  This difference is theregretfor playing an
action, and the accumulated regret is used to determine the
strategy used on the next iteration.  In the limit, the aver-
age strategies used by the players will converge to a Nash
equilibrium.  CFR is efficient in both time and memory, re-
quiring space which is linear in the number of actions across
all  information  sets.   While  it  has  been  applied  to  games
with up to 8.8×10
## 10
information sets [9], the computation
remains intractable for domains as large as two-player limit
Texas hold’em, which has 3.2×10
## 14
information sets.
State-space abstraction.Astate space abstractionis a
many-to-one mapping between the game’s information sets
and the information sets in a smaller, artificially constructed
abstract  game.   An  agent  using  abstraction  only  observes
its abstract game information set, and its strategy for that
information  set  is  used  for  all  of  the  real  information  sets
mapped to it.  The goal is to construct a game small enough
that an optimal strategy can be found through an algorithm
such  as  CFR,  and  the  resulting  strategy  can  be  used  to
choose  actions  in  the  original  game,  where  it  is  hoped  to
closely approximate a Nash equilibrium strategy.  The suc-
cess of this approach relies on both the size of the abstract
game (a larger and finer-grained abstract game can lose less
information) and the domain features used to decide which
information sets can be mapped together.
The  earliest  uses  of  state-space  abstraction  in  poker  in-
volved  the  construction  of  abstract  chance  events,  called
binsby Shi and Littman [16],bucketsby Billingset al.[2],
andsignalsby  Gilpin  and  Sandholm  [5],  by  grouping  to-
gether chance events that are similar according to a metric.
As the players’ actions were left unabstracted, the abstract
game resembles the real game except with a coarsened rep-
resentation  of  the  chance  events.   A  common  metric  used
in this early work is a player’sexpected hand strength
(E[HS]). In the final round when all public cards have been
revealed, a player’shand strength (HS)is the probability
that their hand is stronger than a uniform randomly sam-
pled opponent hand.  In the earlier rounds, expected hand
strength (E[HS]) is the expectation of hand strength over
all possible rollouts of the remaining public cards.  A related
metric,expected  hand  strength  squared  (E[HS
## 2
## ]),
computes the expectation of the squared hand strength val-
ues, and assigns a relatively higher value to hands with the
potential to improve such as flush-draws or straight-draws.

Theseexpectation-basedmetrics can be used to create
abstract chance events in a number of different ways, such as
bucketing based on expert-chosen ranges ofE[HS] values [2],
bucketing based onE[HS] ranges chosen so as to contain an
equal number of hands (calledpercentile bucketing) [21],
or by merging hands whoseE[HS] values differ by less than
a  threshold  [4].   Additionally,  two  abstraction  techniques
can  benestedby  applying  one  and  then  subdividing  by
the other.  For example, an abstraction might split the pos-
sible hands into five buckets by percentileE[HS
## 2
] and fur-
ther split each into two percentileE[HS] buckets, giving ten
buckets overall.  The Percentile nestedE[HS
## 2
] /E[HS] ab-
straction technique has been well studied by researchers [12,
19] and was used by Hyperborean in the Annual Computer
Poker Competitions from 2007 to 2009.
Gilpinet al.showed that expectation-based metrics have
difficulty distinguishing between hands that have the poten-
tial  to  improve  and  those  that  do  not,  and  that  this  dif-
ference is strategically important [7].  High potential hands
are calleddrawing hands,  which might be weak initially
but have the possibility to become very strong given fortu-
nate chance outcomes later in the game.  Expectation-based
abstraction techniques place these hands into buckets along
with hands that have a similarE[HS] values and no likeli-
hood of improving.  Abstracting these strategically distinct
hands  together  loses  information,  as  an  abstracted  agent
must choose one strategy to handle both cases.  While the
## E[HS
## 2
] metric was designed to address this fault, it was only
a  partial  solution.   Gilpinet al.addressed  this  shortcom-
ing through a multi-passk-means abstraction technique in
which the final round is clustered byE[HS] and each earlier
round was clustered byL
## 2
distance over histograms showing
the  probability  of  transitioning  to  the  next  round’s  buck-
ets [7].  In later work, Gilpin and Sandholm compared these
distribution-awareabstractions to expectation-based ab-
stractions  and  found  that  expectation-based  abstractions
yielded stronger strategies in small abstractions, but are sur-
passed as more buckets are made available [6].
Imperfect Recall.Imperfect recallis a relaxation of per-
fect recall in which agents may “forget” some of the informa-
tion that it has observed.  It is not typically a property of a
real domain (as humans cannot be forced to forget their ob-
servations), but is instead an optional property that can be
used for abstract games.  When creating an imperfect recall
abstraction agents can be forced to discard old observations
that are no longer strategically important, thus merging the
real information sets that differed in this observation.  This
means that an agent may be able to distinguish two informa-
tion sets early in a game, but not distinguish their descen-
dant information sets later in the game.  They will perceive
the game as a directed acyclic graph instead of as a tree.
An example of equal-sized perfect recall and imperfect re-
call abstractions in a poker-like game is shown in Figure 1.
This  game  starts  with  a  chance  event,  C,  which  deals  the
player  a  private  card.   Each  abstraction  coarsens  that  in-
formation and maps it to a bucket, 1 or 2, indicating that
the card is in the top or bottom half of all possible cards.
At  the  action  node,  A,  the  players  take  a  sequence  of  ac-
tions, X, or Y, which is followed by a chance node at which
a public card is revealed.  This is where the two abstractions
differ.  In the perfect recall abstraction, the agent must re-
member its sequence of observations:  1 or 2, X or Y. The
new  chance  information  is  coarsened  by  the  perfect  recall
## C
## AA
## CCCC
## 12
## XY
## 1X11X21Y11Y22X12X12Y12Y2
## C
## AA
## CC
## 12
## X
## Y
## *X1*X2*X3*X4*Y1*Y2*Y3*Y4
## 12
## XY
## 121212
## X
## Y
## (a) Perfect Recall
## (b) Imperfect Recall
Figure 1:  Perfect recall and imperfect recall games.
abstraction, and the agent receives one of two new buckets
depending on their earlier observation. The sequences 1-1, 1-
2, 2-1, and 2-2 represent different sets of chance events, and
can  have  overlapping  ranges  according  to  metrics  such  as
E[HS]:  a weak hand that becomes strong may score higher
than  a  strong  hand  that  becomes  weak.   In  the  imperfect
recall  abstraction,  only  the  players’  action  sequence,  X  or
Y, is remembered while the original chance bucket, 1 or 2,
is forgotten.  The 1X and 2X paths merge, as do 1Y and 2Y.
The second chance node is coarsened to one of four buckets,
1  to  4,  representing  the  strength  of  its  private  card  com-
bined with the public card.  These four buckets can be con-
structed to form non-overlapping ranges ofE[HS].  If this
second  chance  event  makes  the  first  less  significant  (i.e.if
the  agent’s  previous  strength  is  not  very  important,  as  is
the case in poker), then the imperfect recall representation
may provide more useful information.
The use of imperfect recall abstractions in the poker do-
main was first proposed by Waughet al.[19]. As they noted,
imperfect recall presents several theoretical challenges: there
is  no  guarantee  that  a  Nash  equilibrium  for  an  imperfect
recall  game  can  be  represented  as  a  behavioral  strategy
(Nash’s  celebrated  theorem  only  guarantees  that  a  mixed
strategy  equilibrium  exists),  and  no  proof  that  CFR  (or
other efficient algorithms) will converge towards such a strat-
egy if one exists.  Recent work by Lanctotet al.has shown
that CFR will converge in a class of imperfect recall games;
however,  this class does not include the abstractions typi-
cally used in poker [13].  However, CFR remains well-defined
in imperfect recall  abstractions and can be used to gener-
ate  abstract  strategies  that  can  be  used  in  the  real  game.
Waughet al.[19] showed that a small improvement was pos-
sible in two-player limit Texas hold’em, as imperfect recall
discarded less relevant earlier observations and allowed new
domain features to be used along withE[HS].
## 3.  EVALUATING ABSTRACTIONS
With many options available for constructing abstractions
and no theory to guide these choices, progress has only been
established through empirical evaluation.  This involves cre-
ating abstract games, solving them, and evaluating the strat-
egy  in  the  real  game.   Gilpin  and  Sandholm  [6]  codified

the  possibilities  for  evaluating  the  resulting  strategy,  and
thus  evaluating  the  abstraction  methodology  itself.   They
described  three  approaches:   in-game  performance  against
other agents, in-game performance against an unabstracted
Nash equilibrium, and exploitability in the real game.
While  these  evaluation  methods  measure  qualities  we
want, each involves a potentially serious drawback.  In one-
on-one play, it is possible to find intransitivities where strat-
egyAdefeatsB, which defeatsC, which defeatsA. A weaker
form of intransitivity occurs whenAdefeatsB, butBdefeats
Cby more thanAdefeatsC. It is not clear what to con-
clude in such cases.  In one-on-one play against a Nash equi-
librium,  many  strategies  of  varying  exploitability  may  tie.
Even more problematic is that generating an unabstracted
equilibrium strategy in human-scale domains is intractable.
Finally, while measuring the exploitability of abstract strate-
gies  directly  addresses  the  goal  of  approximating  a  Nash
equilibrium, recent research has shown that abstract game
equilibria may not be the abstract strategies with the lowest
real game exploitability [18, 12].  In addition, both Waugh
et al.[17, p.30 and p.52] (in a toy game) and Johansonet
al.[12] (in Texas Hold’em),  found that exploitability does
not correlate well with one-on-one performance.
Johansonet al.recently presented CFR-BR: a CFR vari-
ant  that,  in  perfect  recall  abstractions,  converges  towards
an abstract strategy with the lowest real game exploitabil-
ity [10].  These strategies are not abstract game equilibria,
as are found by CFR, but instead are the closest approxi-
mations to a real game equilibrium that can be represented
within an abstraction. In practice, the exploitability of these
CFR-BR strategies is as little as
## 1
## 3
of those found via CFR.
While CFR-BR’s convergence is only proven for perfect re-
call  abstractions,  in  practice  the  same  degree  of  improve-
ment is shown in imperfect recall games.  CFR-BR requires
repeated traversals of the real game tree,  and may not be
tractable in large domains where abstraction enables CFR.
However, calculating a strategy’s exploitability also requires
a  real  game  tree  traversal  (although  an  efficient  traversal
may be possible [12]), and in such large games one-on-one
performance may remain the only viable evaluation.
Johansonet al.also demonstrated that CFR-BR could be
used for evaluating abstractions by measuring the closest ap-
proximation to a Nash equilibrium that can be represented
by the abstraction [10].  In this paper, we will broadly apply
the  CFR-BR  technique  for  the  first  time  to  compare  new
and existing abstraction techniques.  Our experiments will
evaluate two abstraction choices that have been raised by re-
cent publications:  the effectiveness of expectation-based as
opposed to distribution-aware abstractions, as proposed by
Gilpin and Sandholm [7], and the use of perfect recall as op-
posed to imperfect recall, as proposed by Waughet al.[19].
We will also present for the first time the abstraction tech-
nique and distance metrics used by the Hyperborean agent
in the Annual Computer Poker Competition since 2010.
## 4.  ABSTRACTION AS CLUSTERING
To eliminate the need for direct human expert knowledge
when  creating  an  abstraction,  the  abstraction  generation
problem will be considered as a clustering problem.  Given
a target number of clusters (i.e.buckets)kand a distance
function  between  information  sets,  a  clustering  algorithm
can be used to partition the real information sets into the
buckets that form the information sets of the abstract game.
## 0
## 0.05
## 0.10
## 0.15
## 0.20
## 0
## 0.05
## 0.10
## 0.15
## 0.20
## 00.20.40.60.800.20.40.60.81
End of game Hand Strength after rolling out unseen cards
## Histogram Bar Frequency
## 4s4h
## 0.570
## 6s6h
## 0.633
QsKs
## 0.634
TsJs
## 0.575
Earth mover’s DistanceE[HS] Distance
4s4h6s6hTsJsQsKs4s4h6s6hTsJsQsKs
## 4s4h3.1016.2125.1430.0630.0050.064
## 6s6h3.1016.4625.2860.0630.0150.001
TsJs6.2126.4623.1030.0050.0150.059
QsKs5.1435.2863.1030.0640.0010.059
Figure  2:   (top)  Hand  Strength  histograms  for  four  poker
hands  at  the  start  of  the  game.   (bottom)  Earth  mover’s
and E[HS] distances.
Using a clustering algorithm allows the abstraction designer
to  focus  on  two  aspects  of  the  task:  designing  a  distance
metric that represents the strategic similarity of two infor-
mation sets,  and choosing the  total  number  of  clusters on
each round,k
i
, so that the total number of information sets
in the resulting abstract game is small enough to solve.
In  practice,  the  number  of  information  sets  to  be  clus-
tered can be very large, making the use of many clustering
algorithms  computationally  intractable.   In  the  poker  do-
main,  for  example,  the  final  round  of  Texas  hold’em  has
2,428,287,420 canonical combinations of public and private
cards  to  be  grouped  into  between  one  thousand  (a  small
abstraction)  and  one  million  (a  large  abstraction)  clusters
or more.  To make this large clustering problem tractable,
we use ak-means implementation that uses the triangle in-
equality to reduce the number of distance function calls [3].
Multiple restarts and thek-means++ initialization [1] are
also used to improve the quality of the clustering.
As in previous work in the limit Texas hold’em poker do-
main,  the  abstractions  used  in  our  experiments  will  only
merge information sets on the basis of having similar chance
events.    This  approach  leaves  the  players’  actions  unab-
stracted and reduces the abstraction generation task to that
of finding clusters of similar private and public cards.  In the
remainder of this section, we will present two new distance
metrics  for  the  poker  domain  that  capture  strategic  simi-
larities that were not handled by earlier expectation-based
approaches, and describe how imperfect recall can be used to
reallocate the distribution of buckets throughout the game.
Hand Strength Distributions.In Section 2, we described
the expected hand strength metric.  In the final round of the
game,  hand  strength  measures  the  probability  of  winning
against a randomly sampled opponent hand, given the pub-
lic cards.  Earlier in the game,E[HS] measures the expecta-
tion of hand strength over all possibilities for the remaining
public cards.  Thus,E[HS] summarizes thedistributionover
possible end-game strengths into a single expected value.
As  noted  by  Gilpin  and  Sandholm  [6],  this  single  value
is  unable  to  distinguish  hands  with  differing  potential  to
improve.   Consider  Figure  2(top),  which  shows  the  distri-

butions  over  the  final  round  hand  strength  of  four  Texas
hold’em poker hands in the first round of the game.  Each
distribution is discretized into a histogram with values rang-
ing from 0 (a guaranteed loss) to 1 (a guaranteed win).  The
height of each bar indicates the probability of the remaining
public cards resulting in that hand strength, and the vertical
black line and label showsE[HS].
Note that the top and bottom histograms have different
distribution  shapes:   4♠4♥and  6♠6♥have  most  of  their
weight near theirE[HS] values,  whileT♠J♠andQ♠K♠
have almost no weight nearE[HS] as the unrevealed cards
will make this hand either strong or weak.  This difference
is an indication that the top and bottom rows are strategi-
cally distinct:  the bottom row has high potential, while the
top row does not.  However,  when comparing the columns
of hands we find almost identicalE[HS] values.   As such,
expectation-based approaches would merge within each col-
umn, whereas merging along each row may be better.
This  suggests  the  use  of  adistribution-awaresimilar-
ity  metric  such  as  earth  mover’s  distance  [20]  to  compare
two  hand  strength  distributions.   Earth  mover’s  distance
measures  the  minimum  work  required  to  change  one  dis-
tribution  into  another  by  moving  probability  mass.In
one-dimensional  discrete  distributions  such  as  these  hand
strength  distributions,  it  can  be  efficiently  computed  with
a  single  pass  over  the  histogram  bars.   Unlike  alternative
distance metrics such asL
## 2
or Kolmogorov-Smirnov, earth
mover’s distance measures not only the difference in prob-
ability  mass,  but  also  how  far  that  mass  was  moved.   In
Figure 2(bottom), the earth mover’s distance and difference
inE[HS]  for  four  hands  are  listed.   In  partitioning  these
four hands into two clusters, earth mover’s distance would
merge  the  rows  (similar  distribution  shapes)  whileE[HS]
would merge the columns (similar expected values).
In Texas hold’em poker, hand strength histograms can be
precomputed  for  every  combination  of  private  and  public
cards in the first three rounds, and earth mover’s distance
provides a candidate distance function for comparing them.
After all of the public cards are revealed in the final round,
each histogram would be a single impulse at the correspond-
ing hand strength value, and earth mover’s distance and the
difference in hand strength values would be equivalent.
Opponent Cluster Hand Strength.Our second new dis-
tance  metric  addresses  a  different  aspect  ofE[HS].   The
‘hand  strength’  component  ofE[HS]  measures  the  proba-
bility of winning against a uniform randomly sampled oppo-
nent hand at the end of the game, and this provides one sum-
mary feature.  However, we can also consider our probability
of winning against multiple subsets or distributions of pos-
sible opponent hands, and thereby generate additional fea-
tures.  While any number of overlapping or non-overlapping
subsets could be used, in this work we will partition the 169
starting hands into eight non-overlapping subsets, which we
callopponent clusters
## 2
.  These were formed by clustering
the  hands  using  the  earth  mover’s  distance  metric  on  the
first round, and are presented in Table 1.
Instead of using a singleE[HS] value, we will now com-
pute  eight  values  measuring  the  hand  strength  against
hands  drawn  uniform  randomly  from  each  opponent  clus-
ter.   For  example,  the  eighthOpponent  Cluster  Hand
## 2
Our use of these eight clusters was an engineering decision
to  limit  the  memory  required  for  the  precomputed  tables;
other choices may be even more effective.
## Suited
## 2    3    4    5    6    7    8    9    T    J    Q   K   A
## Unsuited
## 24111112224446
## 31611112234466
## 41161122234466
## 51116333334466
## 61111733334566
## 71112373355567
## 81122338355567
## 92222333855577
## T2222333585777
## J2244445558777
## Q4444445555877
## K4446666677787
## A6666666777778
Table 1:  Eight hand clusters used for the OCHS features.
## 4s4h
## 0.57
## 6s6h
## 0.633
QsKs
## 0.634
TsJs
## 0.575
0.7020.6350.4960.6140.4820.5580.4760.1800.7990.6910.5760.6690.4980.6390.4980.183
0.6590.7120.6740.5490.5480.4650.3870.3180.6460.6760.6380.7240.6820.5490.4970.389
## 0
## 0.2
## 0.4
## 0.6
## 0.8
## 1.0
## 0
## 0.2
## 0.4
## 0.6
## 0.8
## 1234567812345678
E[HS] against opponent hands in cluster
Opponent hand clusters
## OCHSL
## 2
DistanceE[HS] Distance
4s4h6s6hTsJsQsKs4s4h6s6hTsJsQsKs
## 4s4h0.0660.0950.1040.0630.0050.064
## 6s6h0.0660.1170.1050.0630.0150.001
TsJs0.0950.1170.0980.0050.0150.059
QsKs0.1040.1050.0980.0640.0010.059
Figure  3:  (top)  OCHS  values  for  four  poker  hands  at  the
start of the game.  (bottom) OCHSL
## 2
and E[HS] distances.
Strength (OCHS)feature measures the probability of win-
ning against an opponent hand sampled from the set of top
pairs.  For each game round, we can precompute a vector of
OCHS features to describe a hand’s strength.  TheL
## 2
dis-
tance between two vectors is then used as a distance met-
ric.   Figure  3  shows  an  example  with  the  four  first-round
hands from Table 2 and theL
## 2
distances between their vec-
tors. OCHS provides a richer representation of strength than
E[HS], which can itself be derived from the vector.
Perfect and Imperfect Recall.We will now describe how
clustering can be used to form abstractions with perfect and
imperfect recall.  A perfect recall abstraction is created hi-
erarchically by solving many small clustering problems.  To
start, the first round of the game is clustered intok
## 1
clusters.
In the second round, perfect recall requires that information
sets may only be clustered together if they share the same
sequence  of  observations.   This  means  that  we  must  solve
k
## 1
independent clustering problems, each of which only in-
cludes  those  chance  events  that  are  descendents  of  chance
events clustered together in the first round.  Although each
of these independent clustering problems could assign a dif-
ferent  number  of  clusters,  in  our  experiments  we  use  the
same constantk
## 2
for each.  The hierarchical abstraction gen-
eration continues until the final round in which we have to
solvek
## 1
## ·...·k
n−1
clustering problems, intok
n
clusters each,
for a total ofk
## 1
## ·...·k
n
clusters in the final round.
When creating an imperfect recall abstraction, we simply
cluster  all  of  the  chance  events  without  considering  their
predecessors’  clusters  on  earlier  rounds.   Solving  one  large

clustering  problem  is  more  computationally  difficult  than
solving  many  small  ones.   However,  the  larger  number  of
clusters may allow for a more accurate clustering, as there
will  not  be  a  need  for  clusters  with  similar  features  that
differ only by their history.
The key constraint when making an abstraction is not the
number of buckets either in each round or overall, but the to-
tal number of information sets in the resulting game, as this
determines the memory required to solve it.  In imperfect re-
call abstractions it is possible to change the distribution of
buckets  throughout  the  game,  dramatically  increasing  the
number  of  buckets  in  early  rounds,  without  changing  the
overall  number  of  information  sets.   We  demonstrate  this
effect in Table 2.  The ‘Action Sequences’ columns describe
only  the  players’  actions  and  not  the  chance  events,  and
shows  the  number  of  action  sequences  leading  to  a  choice
inside  the  round  and  continuing  to  the  next  round.   The
next  three  sections  describe  nearly  equally  sized  abstrac-
tions.  PR-10-10-10-10 uses perfect recall, while IR-10-100-
1,000-10,000 and IR-169-9,000-9,000-9,000 use imperfect re-
call.   For  each  abstraction,  the  table  lists  the  number  of
buckets and the number of information sets (buckets times
decision points) in the abstraction in that round.  The final
row shows the total number of information sets.
The PR-10-10-10-10 and IR-10-100-1,000-10,000 abstract
games  are  exactly  the  same  size  and  use  the  same  total
number  of  buckets  on  each  round:   either  through  multi-
ple small perfect recall clusterings, or in one large imperfect
recall clustering.  The IR-169-9,000-9,000-9,000 abstraction
changes the distribution of buckets, shrinking the final round
to 9,000 buckets and removing 5.67 million final round in-
formation sets.  Due to the multiplying effect of the number
of action sequences that reach the final round, removing one
fourth-round bucket allows for the addition of 9 third-round
buckets, 81 second-round buckets, or 567 first-round buck-
ets. In this way, we can decrease the number of fourth-round
buckets by 10% to get an abstraction that is lossless in the
first round (i.e.it has 169 buckets) and has 9,000 buckets in
the second and third rounds.  Note that this type of redistri-
bution is not possible when using perfect recall, as the larger
number of buckets early in the game need to be remembered
until the final round:  having 169 buckets in the first round
would allow only four buckets on each subsequent round.
## 5.  RESULTS
We can now begin our empirical investigation of abstrac-
tion techniques, using the domain of two-player limit Texas
hold’em poker.  In this paper, we have described three ab-
straction  techniques  that  are  applicable  to  the  first  three
rounds:   Percentile  Hand  Strength  (PHS),k-means  earth
mover (KE), andk-means OCHS (KO). We have two choices
of  abstraction  techniques  to  use  on  the  final  round:   Per-
centile  Hand  Strength  (PHS)  andk-means  OCHS  (KO).
Each combination of an early-game and end-game technique
can  be  used  to  form  a  different  abstraction.   Additionally,
we can consider abstractions that use Perfect Recall (PR)
and Imperfect Recall (IR), resulting in 2×3×2 = 12 ab-
stractions.    An  abstraction  (or  agent)  named  IR-KE-KO
uses imperfect recall,k-means earth mover to abstract the
first three rounds, andk-means OCHS to abstract the final
round.   Each  abstraction  will  be  of  the  sizes  listed  in  Ta-
ble 2:  either Perfect Recall 10-10-10-10, or Imperfect Recall
169-9000-9000-9000  with  a  lossless  first-round  abstraction.
In the first three rounds, PHS abstractions will use nesting
to  partition  first  byE[HS
## 2
]  and  then  byE[HS].   Perfect
recall PHS will use 5×2 = 10 buckets and imperfect recall
PHS will use 150×60 = 9000 buckets.  On the final round
## E[HS
## 2
]  ranks  hands  in  the  same  order  asE[HS],  and  so
PHS uses a single partition into 10 or 9000 buckets.
We  begin  our  evaluation  of  these  abstraction  styles  and
distance  metrics  with  the  first  evaluation  technique  sug-
gested  by  Gilpin  and  Sandholm:   one-on-one  performance
between abstract game Nash equilibrium strategies [6].  For
each  abstraction,  a  parallel  implementation  of  the  Public
Chance Sampled CFR algorithm (PCS) [11] was run for 4
days on a 48-core 2.2 GHz AMD computer
## 3
.  Each pair of
strategies was then played against each other for 10 million
hands  of  duplicate  poker  to  obtain  statistically  significant
results  with  a  95%  confidence  interval  of  1.1  mbb/g.   The
crosstable of this match is shown in Table 3.  We find that
every imperfect recall agent, regardless of abstraction tech-
nique, outperformed every perfect recall agent.  Comparing
each imperfect recall agent against its perfect recall equiv-
alent (i.e., IR-KE-KO to PR-KE-KO) we find that the im-
perfect recall agent also had a higher expected value against
each  opponent.   Overall,  the  IR-KE-KO  agent  was  unde-
feated and additionally scored the highest against each ad-
versary.   Ranked  by  average  performance,  the  IR-KO-KO
and IR-KE-PHS agents placed second and third.
Gilpin and Sandholm’s third abstraction evaluation tech-
nique is to calculate the real game exploitability of abstract
game Nash equilibrium strategies. In the CFR column of Ta-
ble 4, we present the exploitability of the same CFR strate-
gies used in the one-on-one crosstable.  Note that the results
are  inconsistent:   neither  perfect  recall  or  imperfect  recall
shows a clear advantage. Notably, the two KE-KO strategies
are almost exactly tied, despite the fact that IR-KE-KO was
considerably stronger in the crosstable.  As described earlier,
recent work by Waughet al.[18] and Johansonet al.[12]
has shown that abstract game Nash equilibria are rarely the
least exploitable strategies representable in an abstraction,
making this method of evaluating abstractions inconclusive.
The  recently  developed  CFR-BR  algorithm  provides  a
more  reliable  metric  [10].   In  each  abstraction,  a  parallel
implementation of CFR-BR was run for 8 days on the same
computer  used  to  generate  the  CFR  strategies
## 4
.   The  ex-
ploitability of these CFR-BR strategies is presented in Ta-
ble  4,  and  the  results  are  much  more  consistent  with  the
one-on-one performance presented in Table 3.  IR-KE-KO,
IR-KO-KO,  and  IR-KE-PHS  are  once  again  ranked  first,
second  and  third.   With  the  exception  of  PHS-PHS,  the
imperfect  recall  agents  are  also  less  exploitable  than  their
perfect recall equivalents.  Johansonet al.note that CFR-
BR strategies tend to lose slightly when played against their
more exploitable PCS equivalents [10,  Fig.   8],  and so the
CFR strategies’ one-on-one performance is of more interest.
The outcomes of playing the CFR-BR agents against each
other are very similar to those of the CFR agents in Table 3.
In Table 2, we showed that imperfect recall allows us to
decrease the number of buckets in later rounds of the game
## 3
Johansonet al.found that applying PCS to 10-bucket PR-
PHS-PHS  for  10
## 5
seconds  was  sufficient  for  near  conver-
gence [11, Figure 3c].
## 4
Johansonet al.found  that  this  time,  3.3×10
## 7
seconds,
was sufficient for near convergence using PR-PHS-PHS and
IR-KE-KO [10, Figures 6 and 7].

# Action SequencesPR 10-10-10-10IR 10-100-1,000-10,000IR 169-9,000-9,000-9,000
RoundInsideContinuing# Buckets# Infosets# Buckets# Infosets# Buckets# Infosets
## 187108010801691,352
## 27*107*910*107001007009,000630,000
## 37*9*107*9*910*10*10630,0001,000630,0009,0005,670,000
## 47*9*9*1010*10*10*1056,700,00010,00056,700,0009,00051,030,000
## Total57,330,78057,330,78057,331,352
Table 2:  Computing the number of information sets in three nearly equally sized Texas hold’em abstractions.
Perfect RecallImperfect RecallMean
PHS-PHS   PHS-KO   KE-PHS   KE-KO   KO-PHS   KO-KOPHS-PHS   PHS-KO   KE-PHS   KE-KO   KO-PHS   KO-KO
## PR
## PHS-PHS-0.9-1.9-2.714.314.5-8.8-14.8-21.1-25.7-19.1-22.4-8.1
## PHS-KO0.9-0.3-2.314.213.5-8.7-14.7-21.0-26.3-17.6-22.2-7.7
## KE-PHS1.90.3-2.415.214.5-6.5-13.0-18.0-24.2-16.1-21.6-6.4
## KE-KO2.72.32.416.916.3-6.6-12.5-18.6-24.8-15.8-21.0-5.3
## KO-PHS-14.3-14.2-15.2-16.9-1.69-25.7-30.3-34.9-39.9-32.6-37.3-23.9
## KO-KO-14.5-13.5-14.5-16.31.69-25.1-30.2-34.8-39.6-31.8-37.2-23.3
## IR
## PHS-PHS8.88.76.56.625.725.1-5.5-15.5-18.9-10.8-15.01.4
## PHS-KO14.814.713.012.530.330.25.5-9.6-14.6-6.0-10.47.3
## KE-PHS21.121.018.018.634.934.815.59.6-6.72.6-3.815.1
## KE-KO25.726.324.224.839.939.618.914.66.79.32.821.2
## KO-PHS19.117.616.115.832.631.810.86.0-2.6-9.3-7.211.9
## KO-KO22.422.221.621.037.337.215.010.43.8-2.87.217.8
Table 3:  Average performance in games between abstract strategies generated by Public Chance Sampled CFR. Results are
in milli-big-blinds/game (mbb/g) over a 10 million hand duplicate match with a 95% confidence interval of 1.1 mbb/g.
## CFRCFR-BR
## PRIRPRIR
## PHS-PHS288.942358.18889.63294.841
## PHS-KO289.152318.19790.37185.275
## KE-PHS281.63339.87290.72080.557
## KE-KO282.856282.39584.03964.820
## KO-PHS335.902355.881105.38988.546
## KO-KO330.319291.574105.52373.091
Table 4: Exploitability of CFR-BR and CFR strategies.  Re-
sults are measured in milli-big-blinds/game and are exact.
AbstractionCFR-BR Exploitability
## PR KE-KO 10-10-10-1084.039
## IR KE-KO 10-100-1000-1000089.7975
## IR KE-KO 169-9000-9000-900064.820
Table 5:  Effect of redistributing buckets in an abstraction.
in  return  for  many  more  buckets  in  earlier  rounds,  with-
out increasing the size of the game.  In Table 5, we revisit
this decision and also consider an IR KE-KO 10-100-1,000-
10,000 abstraction.  We find that the imperfect recall agent
is more exploitable than its perfect recall equivalent, while
the redistributed agent shows a significant decrease.
We can also measure the exploitability of CFR-BR strate-
gies as a response to abstraction size, to investigate if these
abstraction techniques improve at different rates.  For this
experiment, we consider five sizes of four abstractions:  PR-
PHS-PHS  and  IR-PHS-PHS,  PR-KE-KO  and  IR-KE-KO.
The  perfect  recall  abstractions  branch  to  5,  6,  8,  10  and
12 buckets on each round, and the imperfect recall abstrac-
tions have a lossless first round and 570,  1175,  3700,  9000
and 18630 buckets on later rounds.
The CFR-BR exploitability results for these abstractions
are presented in Figure 4 as a log-log plot.  Comparing the
slope of each curve, we find that IR-KE-KO and PR-KE-KO
are steeper than PR-PHS-PHS and IR-PHS-PHS, indicating
that their advantage increases with the abstraction size. The
combination of abstraction techniques presented in this pa-
per, imperfect recall with redistribution and the KE and KO
techniques, is less exploitable at all tested abstraction sizes.
## 10
## 2
## 10
## 6
## 10
## 7
## 10
## 8
## 10
## 9
## Exploitability (mbb/g)
Abstraction Size (# information sets)
## PR-PHS-PHS
## IR-PHS-PHS
## PR-KE-KO
## IR-KE-KO
Figure  4:  Exploitability  of  CFR-BR  strategies  in  four  ab-
stractions as the abstraction size is varied.
## 6.  DISCUSSION
Recent  research  towards  state-space  abstraction  in  the
poker  domain  has  raised  two  issues:   the  effectiveness  of
distribution-aware  as  compared  to  expectation-based  ap-
proaches (as described by Gilpin and Sandholm [6]) and the
practical uses of imperfect recall (as described by Waughet
al.[19]).   The discovery that the exploitability of abstract
game Nash equilibrium strategies was not an accurate mea-
sure of an abstraction’s ability to represent a real Nash equi-
librium has left these issues unresolved. Our goal in these ex-
periments was to use the recently developed CFR-BR tech-
nique to survey these abstraction choices and evaluate them
more precisely.
Gilpin  and  Sandholm’s  investigation  showed  that  while
agents  in  expectation-based  abstractions  are  more  effec-
tive in small abstract games, the distribution-aware agents
match  and  surpass  them  as  the  number  of  buckets  is  in-
creased.  Figure 4 shows that our experiment matches their
result: the steeper slope of the PR-KE-KO line as compared
to  PR-PHS-PHS  shows  that  the  distribution-aware  metric
makes  better  use  of  the  available  buckets  as  the  abstrac-
tion  size  increases.   In  addition,  the  one-on-one  crosstable
in  Table  3  shows  that  the  distribution-aware  agents  using
thek-means  earth  mover’s  abstractions  outperformed  the
expectation-based agents.

We  now  turn  to  imperfect  recall.   In  one-on-one  perfor-
mance,  every  imperfect  recall  agent,  regardless  of  its  ab-
straction features, outperformed every perfect recall agent.
In  terms  of  exploitability,  aside  from  IR-PHS-PHS,  every
CFR-BR agent using imperfect recall except was found to
be less exploitable than its perfect recall equivalent.  While
CFR-BR  is  not  theoretically  guaranteed  to  converge  to  a
least exploitable strategy in an imperfect recall game,  our
results  provide  an  upper  bound:  the  least  exploitable  IR-
KE-KO  strategy  is  exploitable  for  at  most  64.820  mbb/g,
far less than the least exploitable perfect recall agent.  While
Waughet al.found that imperfect recall and additional fea-
tures provided a small advantage, we have shown a signifi-
cant improvement while using the same domain features.
Finally,  the  CFR  and  CFR-BR  results  presented  in  Ta-
ble  4  support  Johansonet al.’s  proposed  use  of  CFR-BR
to evaluate strategies instead of measuring the exploitabil-
ity of abstract game Nash equilibria.  The CFR results are
inconsistent, showing no clear advantage for perfect or im-
perfect recall, and ordering the agents differently than the
one-on-one crosstable.  While there is no guarantee that the
one-on-one results and exploitability should agree, the CFR-
BR strategies are both far less exploitable in all cases, show
an  advantage  for  imperfect  recall,  and  rank  the  top  three
agents in the same order as the one-on-one results.  Using
CFR-BR to evaluate abstractions based on their ability to
approximate  an unabstracted Nash equilibrium provides  a
more consistent metric than the previous approaches.
## 7.  CONCLUSION
## Historically,state-spaceabstractiontechniquesin
extensive-form  games  have  been  evaluated  by  computing
optimal abstract strategies and comparing their one-on-one
performance   and   exploitability.A   recently   published
technique,  CFR-BR,  directly  finds  the  abstract  strategy
with the lowest real game exploitability,  providing a more
consistent  measure  of  an  abstraction’s  quality.   Using  this
technique, we evaluated two abstraction choices in the poker
domain: expectation-based as opposed to distribution-aware
distance  metrics,  and  imperfect  recall  abstractions.    Our
findings  on  distribution-aware  techniques  support  those  of
Gilpin  and  Sandholm:  distribution-aware  distance  metrics
provide  a  clear  advantage  once  the  abstract  game  is  large
enough.We  also  demonstrated  a  clear  improvement  in
one-on-one performance and exploitability through the use
of  imperfect  recall  abstractions,  and  demonstrated  that
imperfect  recall  abstractions  can  contain  less  exploitable
strategies than equal sized perfect recall strategies.
## Acknowledgements
We would like to thank Mihai Ciucu, Eric Jackson, Mengliao
Wang, and the members of the University of Alberta Com-
puter Poker Research Group.  This research was supported
by NSERC and Alberta Innovates Technology Futures, and
was made possible by the computing resources provided by
WestGrid,  R ́eseau  Qu ́eb ́ecois  de  Calcul  de  Haute  Perfor-
mance, and Compute/Calcul Canada.
## 8.  REFERENCES
[1]  D. Arthur and S. Vassilvitskii.k-means++:  The
advantages of careful seeding. InSODA, 2007.
## [2]  D. Billings, N. Burch, A. Davidson, R. Holte,
J. Schaeffer, T. Schauenberg, and D. Szafron.
Approximating game-theoretic optimal strategies for
full-scale poker. InIJCAI, 2003.
[3]  C. Elkan. Using the triangle inequality to accelerate
k-means. InICML, 2003.
[4]  A. Gilpin and T. Sandholm. A competitive texas
hold’em poker player via automated abstraction and
real-time equilibrium computation. InAAAI, 2006.
[5]  A. Gilpin and T. Sandholm. Lossless abstraction of
imperfect information games.Journal of the ACM,
## 54(5), 2007.
[6]  A. Gilpin and T. Sandholm. Expectation-based versus
potential-aware automated abstraction in imperfect
information games:  An experimental comparison using
poker. InAAAI, 2008.
[7]  A. Gilpin, T. Sandholm, and T. B. Sørensen.
Potential-aware automated abstraction of sequential
games, and holistic equilibrium analysis of texas
hold’em poker. InAAAI, 2007.
[8]  S. Hoda, A. Gilpin, J. Pe ̃na, and T. Sandholm.
Smoothing techniques for computing Nash equilibria
of sequential games.Mathematics of Operations
## Research, 35(2):494–512, 2010.
[9]  E. Jackson. Slumbot:  An implementation of
counterfactual regret minimization on commodity
hardware. In2012 Computer Poker Symposium, 2012.
[10]  M. Johanson, N. Bard, N. Burch, and M. Bowling.
Finding optimal abstract strategies in extensive-form
games. InAAAI, 2012.
[11]  M. Johanson, N. Bard, M. Lanctot, R. Gibson, and
M. Bowling. Efficient Nash equilibrium approximation
through Monte Carlo counterfactual regret
minimization. InAAMAS, 2012.
[12]  M. Johanson, K. Waugh, M. Bowling, and
M. Zinkevich. Accelerating best response calculation in
large extensive games. InIJCAI, pages 258–265, 2011.
[13]  M. Lanctot, R. Gibson, N. Burch, and M. Bowling.
No-regret learning in extensive-form games with
imperfect recall. InICML, 2012.
[14]  N. A. Risk and D. Szafron. Using counterfactual
regret minimization to create competitive multiplayer
poker agents. InAAMAS, 2010.
[15]  T. Sandholm. The state of solving large
incomplete-information games, and application to
poker.AI Magazine, 31(4):13–32, 2010.
[16]  J. Shi and M. L. Littman. Abstraction methods for
game theoretic poker. InComputers and Games, 2000.
[17]  K. Waugh. Abstraction in large extensive games.
Master’s thesis, University of Alberta, 2009.
[18]  K. Waugh, D. Schnizlein, M. Bowling, and D. Szafron.
Abstraction pathology in extensive games. In
## AAMAS, 2009.
## [19]  K. Waugh, M. Zinkevich, M. Johanson, M. Kan,
D. Schnizlein, and M. Bowling. A practical use of
imperfect recall. InSARA, 2009.
[20]  Wikipedia. Earth mover’s distance — Wikipedia, the
free encyclopedia, 2013.
[21]  M. Zinkevich, M. Johanson, M. Bowling, and
C. Piccione. Regret minimization in games with
incomplete information. InNIPS, 2008.