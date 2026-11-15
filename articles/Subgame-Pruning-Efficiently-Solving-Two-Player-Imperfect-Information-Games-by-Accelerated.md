

## IEEE TRANSACTIONS ON GAMES, VOL. 18, NO. 2, JUNE 2026411
Subgame Pruning: Efficiently Solving Two-Player Imperfect Information Games
by Accelerated Public Tree Traversal in CFR
## Shenkai Zhang
, Peipei Yang, Zekeng Zeng, and Junge Zhang, Member, IEEE
Abstract—Counterfactual regret minimization (CFR) is the state-of-the-
art algorithm for solving imperfect information games, yet it struggles
against scalability. While existing pruning techniques primarily focus on
pruning unreachable branches, they fail to account for subgames, in which
strategies have temporarily converged. Continuing to update strategies
within these subgames leads to unnecessary computational overhead. To
address this, we propose Subgame Pruning, a novel regret-based pruning
paradigm that accelerates traversal of the public tree by dynamically prun-
ing subgames where strategies remain stable across iterations. To ensure
safe and efficient pruning, we introduce two key mechanisms: pruning
constraint checking, which verifies whether a subgame satisfies pruning
criteria, and regret matching compensation, which defers regret matching
and average strategy updates until the pruned subgames are revisited. Our
method significantly reduces computational overhead while preserving the
theoretical convergence guarantees of CFR. In particular, the exploitability
of the average strategy profile remains bounded byO(1/
## √
## T). Experimen-
tal results across five benchmark games demonstrate substantial reductions
in the number of traversed nodes, with exploitability comparable to that of
standard CFR.
Index Terms—Counterfactual regret minimization (CFR), imperfect
information games, regret-based pruning (RBP).
## I. INTRODUCTION
Imperfect information extensive-form games are extensive-form
games in which each player’s decision nodes are partitioned into
information sets[13]. It is a powerful tool for modeling multiagent
sequential decision processes with hidden information, such as ne-
gotiations, auctions, and security problems[14],[15],[16].Ingame
theory, solving an imperfect information game typically entails approx-
imating a Nash equilibrium, a strategy profile from which no player
can unilaterally deviate to improve their expected payoff. However,
the presence of private information, which is not fully observable to
all players, complicates strategy optimization and makes equilibrium
approximation substantially more difficult.
Received 9 May 2025; revised 24 October 2025, 24 January 2026, and 11
March 2026; accepted 14 March 2026. Date of publication 18 March 2026; date
of current version 18 June 2026. This work was supported in part by the National
Key R&D Program of China under Grant 2025ZD0122003 and in part by Beijing
Major Science and Technology Project under Grant Z251100008425009. Rec-
ommended by Associate Editor R. Thawonmas.(Corresponding author: Junge
## Zhang.)
Shenkai Zhang and Zekeng Zeng are with the School of Artificial Intelligence,
University of Chinese Academy of Sciences, Beijing 100049, China, and also
with the National Key Laboratory of Cognition and Decision Intelligence for
Complex Systems, Institute of Automation, Chinese Academy of Sciences,
## Beijing 100190, China.
Peipei Yang is with the State Key Laboratory of Multimodal Artificial Intelli-
gence Systems, Institute of Automation, Chinese Academy of Sciences, Beijing
100190, China, and also with the School of Artificial Intelligence, University of
Chinese Academy of Sciences, Beijing 100049, China.
Junge Zhang is with the National Key Laboratory of Cognition and Decision
Intelligence for Complex Systems, Institute of Automation, Chinese Academy
of Sciences, Beijing 100190, China, and also with the School of Artificial
Intelligence,UniversityofChineseAcademyofSciences,Beijing100049,China
(e-mail: jgzhang@nlpr.ia.ac.cn).
Digital Object Identifier 10.1109/TG.2026.3675377
Counterfactual regret minimization (CFR)[2]is the state-of-the-art
algorithm for solving imperfect information extensive-form games. It
operates by iteratively minimizing each player’s counterfactual regret
at each decision node. In recent years, CFR-based agents have achieved
remarkable success, particularly in solving Texas Hold’em poker[3],
## [4],[5].
Despite its impressive performance, CFR suffers from limited com-
putational efficiency. There are two primary approaches to improving
its efficiency: one involves accelerating the convergence rate of the
algorithm, i.e., reducing regret more effectively with each iteration;
while the other focuses on lowering the computational cost per iteration.
A series of variants of CFR have been proposed to accelerate its
convergence, such as CFR+[6], discounted CFR[7], and balanced
CFR[17]. In contrast, much less attention has been paid to reducing
per-iteration computational overhead, which is the focus of this work.
Pruning is an effective technique for improving the computational
efficiency of CFR. Although prior pruning methods, such as partial
pruning[8]and regret-based pruning (RBP)[9], have significantly
accelerated CFR, they mainly target unreachable or low-probability
branches. For example, RBP prunes subtrees when the current player’s
strategy assigns zero probability to actions leading to them. However,
there remain additional pruning opportunities, even in reachable sub-
trees. In particular, traversing subtrees where strategies have temporar-
ily stabilized often incurs redundant computation. To the best of the au-
thors’ knowledge, such pruning opportunities have not been explicitly
explored by existing methods. Moreover, existing pruning techniques
are less effective during public tree traversal, as they are primarily
designed for the game tree or the information-set tree. In contrast,
public tree traversal improves efficiency by reusing cached strategies
and sharing a set of reach probabilities across different information sets
within the same public node[1].
This article introduces a novel pruning technique, Subgame Pruning
(SP), specifically designed for public tree traversal. While previous
methods focus on pruning unreachable branches, SP targets subgames
in which the strategy profile has temporarily converged, even if those
branches remain reachable. We observe that in certain subtrees of the
publictree,thestrategyprofileinCFRiterationsfrequentlyconverges—
changing slowly or remaining unchanged over multiple iterations.
Intuitively, traversing such regions provides limited benefit, as updates
to stabilized strategies cannot reduce regret any faster. In contrast,
pruning these subgames allows the algorithm to shift computational
focus to regions where regret can still be effectively minimized. Lever-
aging this insight, SP improves the efficiency of CFR by pruning
subgames with converged strategy profiles. Fig.1illustrates the key
difference between prior pruning methods and SP. To integrate SP
into the CFR algorithm, we introduce pruning constraint checking,
which determines whether and when a subgame should be pruned or
reactivated. To ensure consistency with CFR updates while introducing
minimal computational overhead, we further propose regret matching
(RM) compensation, which defers RM and average strategy updates
for pruned subgames, and resumes these updates once the subgames
2475-1510 © 2026 IEEE. All rights reserved, including rights for text and data mining, and training of artificial intelligence and similar technologies.
Personal use is permitted, but republication/redistribution requires IEEE permission. See https://www.ieee.org/publications/rights/index.html for more information.
Authorized licensed use limited to: Rajshahi University Of Engineering and Technology. Downloaded on November 15,2026 at 09:38:45 UTC from IEEE Xplore.  Restrictions apply.

## 412IEEE TRANSACTIONS ON GAMES, VOL. 18, NO. 2, JUNE 2026
Fig. 1.  Comparison of pruning paradigms between existing methods (left)
and SP (right). Previous pruning techniques cut a branch if it is unreachable in
a rollout, while SP cuts a branch if the reach probability to all states inside the
subgame remains unchanged.
are reactivated. Theoretical analysis confirms that CFR’s convergence
guarantees remain valid under SP. In our experiments, applying SP
to CFR and CFR+ yields substantial efficiency gains across several
two-player zero-sum imperfect information games, while preserving
exploitability comparable to the baseline.
Our key contributions are as follows: 1) a novel SP framework for
public tree traversal; 2) a theoretical bound on the additional regret
introduced by SP; and 3) experimental results across benchmark games
demonstrating that SP significantly reduces the number of traversed
nodes while maintaining solution quality.
The rest of this article is organized as follows. SectionIIgives a
brief description of some useful background knowledge of this work,
followed by a brief introduction to related work in SectionIII. Details
of the new pruning technique are introduced in SectionIV, and the
experimental results are presented in SectionV. Finally, SectionVI
concludes this article.
## II. PRELIMINARIES
A. Extensive-Form Games
Extensive-form games offer a compact framework for modeling
multiagent sequential decision-making processes. Such a game can be
represented as a tree, where each node denotes a game state, and each
edge corresponds to an available action. A finite extensive-form game
is defined by the following components.
1)N: The set of strategic players in the game.
2)H: The set of possible histories in the game, which is the standard
notation for game states. Taking actionaat historyhresults in a
successor history, denoted asha,wherehis called the parent of
ha. Historyh
## 
is called an ancestor ofhif there exists a sequence
of actions leading fromh
## 
toh, denoted byh
## 
## h.
3)Z: The set of terminal histories, where the game ends.
4)u: The utility function of the game, determining the payoff
for each player. The maximum payoff difference for player
iacross all outcomes is defined asΔ
i
## =max
z∈Z
u
i
## (z)−
min
z∈Z
u
i
(z), and the global maximum difference isΔ=
max
i∈N
## Δ
i
## .
5)P:H\Z→N∪{chance}: A mapping that identifies the de-
cision maker at each nonterminal history. Histories assigned
to chance involve stochastic transitions whose probabilities are
fixed and publicly known.
## 6)A:H\Z→2
## A
: The set of legal actions available to a player
given the current history, whereArepresents the set of all
possible actions.
Throughout this work, we specifically focus on two-player zero-
sum extensive-form games, possibly including chance moves, in which
## N={1,2}andu
## 1
## =−u
## 2
## .
Depending on whether there is private information that is not visible
to all players, extensive-form games can be divided into perfect in-
formation extensive-form games and imperfect information extensive-
form games.I
i
(h)represents the part of history visible to playeri.
Histories sharing the sameI
i
(h)form an information set. The infor-
mation sets of each player constitute a partition ofH, denoted asI
i
## .
By convention,Idenotes a single information set of any player, with
subscripts omitted for brevity. A game is said to be ofperfect recall
if all players can recall their previous actions. A perfect information
extensive-form game can be regarded as a degenerate case of an
imperfect information extensive-form game where every information
set is a singleton.
A strategy determines how players should act when they encounter
different states of the game.σ
i
## ∈Σ
i
denotes the strategy of playeri,
andσ∈Σ
## 1
## ×...×Σ
n
denotes the strategy profile of all players. As
both the player to act and the set of legal actions are consistent across
all histories within an information setI, the notationsP(I)andA(I)
are reused to denote the player to act and the set of legal actions atI,
respectively. For eacha∈A(I),σ
i
(a|I)is the probability that playeri
would take actionaatI,andσ
i
(I)denotes the behavioral strategy atI,
a distribution overA(I).Reach probabilityrepresents the probability
that a history will be encountered during a rollout, which is indicated
asπ
σ
(h). Playeri’s contribution to the reach probability is represented
byπ
σ
i
(h), while the contribution of all other strategic players together
with chance, excluding playeri, is denoted byπ
σ
## −i
## (h),referredtohere
as theexternal reach probability.
In a finite, perfect-recall extensive-form game, public nodes can be
viewed as the finest partition over all histories that reflects the players’
shared information. A practical way to describe this partition is through
abipartitegraphwhoseverticesconsistofallhistoriesHandallplayers’
information sets
## 
i
## I
i
. An edge connects a historyhand an information
setIwheneverh∈I. Two elements belong to the same public node
if and only if they are connected by a path in this graph. Equivalently,
public nodes correspond to the connected components of this bipartite
structure. These public nodes can be organized into a public tree, where
parent-childrelations followthoseamongthecorrespondinghistories in
the original game tree. This tree provides an intuitive representation of
how publicly observable information evolves during play. As illustrated
in Fig. 1 of Johanson et al.[1], publicly observed actions, including
players’ moves (e.g., raise, call, and fold) and the dealer’s revealed
cards, define the shared information that determines the branching of
the public tree in poker.
## B. Counterfactual Regret Minimization
CFR is the state-of-the-art algorithm for solving extensive-form
games with imperfect information. CFR minimizes a player’s global
regret by reducing regret at each decision node, typically through RM
or its variants. Given strategy profileσ, counterfactual value (CFV) of
playerion information setIis given byv
σ
i
## (I)=
## 
h∈I
π
σ
## −i
## (h)u
σ
i
## (h),
whereu
σ
i
(h)represents the expected utility playeriwill get ath
under strategy profileσ.Theinstantaneous regretfor playeriat
information setIwith actionaunder strategy profileσis defined
asr
σ
i
(I,a)=v
σ
i
(I,a)−v
σ
i
(I).Lettdenote the iteration index of
the CFR algorithm. Theaccumulated regretof taking actionaatI
is the sum of instantaneous regrets over the firstTiterations, i.e.,
## R
## T
i
(I,a)=
## 
## T
t=1
## (v
σ
t
i
(I,a)−v
σ
t
i
(I)). And the accumulated regret
ofIis defined asR
## T
i
(I)=max
a
## R
## T
i
(I,a). The average overall regret
of the game is defined asR
## T
i
## =
## 1
## T
max
σ
## ∗
i
## ∈Σ
i
## 
## T
t=1
## (u
## σ
## ∗
i
## ,σ
t
## −i
## 
i
## −u
σ
t
i
## ),
whereu
σ
i
denotes the expected utility of playeriover the entire game
when all players follow the strategy profileσ, formally defined as
u
σ
i
## =
## 
h∈Z
u
i
## (h)π
σ
(h). Fig.2illustrates the three-step implemen-
tation of the CFR iteration. First, during the reach probability and
Authorized licensed use limited to: Rajshahi University Of Engineering and Technology. Downloaded on November 15,2026 at 09:38:45 UTC from IEEE Xplore.  Restrictions apply.

## IEEE TRANSACTIONS ON GAMES, VOL. 18, NO. 2, JUNE 2026413
Fig. 2.  Illustration of the CFR iteration process at a public node level with SP.p
## 1
denotes the public node being updated, withp
## 2
andp
## 3
representing its child
nodes. If a public node is pruned by SP, the first step of the CFR iteration is skipped, and the second and third steps are deferred until the pruning condition no
longer holds.
utility update step, the reach probabilities and utilities are updated via
forward and backward traversals of the game tree, respectively, using
strategies derived from accumulated regrets. Second, during the average
strategy update step, the average strategy is updated based on the current
strategy and reach probabilities. The average strategy ̄σ
## T
i
of playeri
at information setIis defined as the self-reach-probability-weighted
average of the strategies adopted atIover theTiterations, i.e.,
## ̄σ
## T
i
(a|I)=
## 
## T
t=1
π
σ
t
i
(I)σ
t
i
(a|I)
## 
## T
t=1
π
σ
t
i
## (I)
## .(1)
Finally, in the RM step, CFV are computed, and RM is applied at each
decision node by updating regrets using the CFV of the current and
child nodes.
In RM, a new strategy is generated based on the positive part of the
accumulated regret associated with each action
σ
## T+1
i
## =
## ⎧
## ⎨
## ⎩
## R
## T,+
i
(I,a)
## ∑
a
## R
## T,+
i
(I,a)
## ,if
## 
a
## R
## T,+
i
(I,a)>0
## 1
## |A(I)|
## ,otherwise
## (2)
whereR
## T,+
i
(I,a)=max(R
## T
i
(I,a),0).  The accumulated  regret
## R
## T
i
(I)under RM is bounded byΔ
i
## 
|A(I)|T[11]. In addition, the
sum of the positive parts of accumulated regrets over a player’s decision
informationsetsprovidesanupperboundontheplayer’saverageoverall
regret in the game[2];thatis
## R
## T
i
## ≤
## 1
## T
## 
P(I)=i, I∈I
i
max(R
## T
i
## (I),0).(3)
In a two-player zero-sum game, if the average overall regrets of both
players are smaller than, then the average strategy profile forms a
## 2-equilibrium[2].
## III. RELATEDWORK
This section reviews the pruning techniques that have been applied
in CFR. Partial pruning[8], a game tree pruning technique, cuts the
branches beyond a game statehifπ
σ
## −i
(h)=0. Since the external reach
probability is zero ath, there is no need to compute playeri’s utility
for any game stateh
## 
beyondh. This is because CFVv
i
## (I
i
## (h
## 
## ))always
assigns zero weight to the expected utility ofh
## 
due to the fact that
π
σ
## −i
## (h
## 
)=0for allhh
## 
## .
RBP[9]extends this idea to the information-set tree. It prunes a
subtree beyond an information setIwhen the current player’s reach
probability satisfiesπ
σ
i
(I)=0, meaning the player’s own strategy does
not contribute to reachingI. In such cases, the player’s behavior within
the pruned branches has no effect on the CFR traversal of other players,
asthoseplayerscansafelyapplypartialpruningtocutthecorresponding
branches. Based on RBP and warm starting techniques, Brown and
Sandholm[10]proposed a more aggressive pruning method called Total
RBP. Total RBP not only accelerates convergence, but also significantly
reduces the space requirements of CFR.
Expected value-based pruning (EVP), introduced by Li and
Huang[18], is an offline pruning technique applied prior to CFR. It
## TABLE I
## P
## AYOFFMATRIX FORPLAYER1
permanently prunes suboptimal branches whose expected payoffs, as
predicted by neural networks, are lower than those of others.
Offline pruning techniques, such as EVP, apply pruning before the
CFR process begins, relying on prior knowledge about the game—
typically obtained by solving numerous subgames. While effective at
permanently reducing the search space, these methods incur substantial
offline computational costs. In contrast, online pruning techniques
operate during the CFR process. However, most existing online pruning
methods adopt a static perspective, evaluating pruning opportunities
independently at each iteration. These methods determine whether to
prune a branch of the game or information-set tree based solely on
its immediate effect on CFV computation.
## 1
In contrast, SP adopts a
dynamic perspective, leveraging the evolution of the strategy profile
across iterations. It identifies stable subgames, those whose strategy
profiles exhibit little or no change over time, and prunes them without
affecting the algorithm’s convergence. By reusing intermediate results
from previous iterations, SP avoids redundant updates and improves
efficiency. Moreover, SP operates directly on the public tree, allowing
it to exploit the efficiency of public-tree traversal. In contrast, prior
techniques cannot fully benefit from this efficiency, since their pruning
decisions are made at finer grained representations.
## IV. METHOD
This section introduces a new pruning technique referred to as SP.
Unlike previous methods that primarily prune unreachable branches,
SP prunes subtrees from the public tree when their strategy profiles
exhibit little to no change across iterations, thereby accelerating public
tree traversal.
To intuitively illustrate the underlying mechanism, consider a simple
two-player zero-sum matrix game where Player 1 chooses actionA0
orA1, Player 2 chooses actionB0orB1, and Player 1’s payoff matrix
is shown in TableI. The Nash equilibrium of this game is given by
Player 1 playing(
## 1
## 1.001
## ,
## 0.001
## 1.001
)and Player 2 playing(
## 1
## 2.002
## ,
## 1.002
## 2.002
## ).
When transformed into an imperfect information extensive-form game,
this matrix game can be solved using CFR, and the evolution of the
strategy profile can be visualized. As shown in Fig.3, the trajectory of
the strategy profile moves clockwise along the boundary and tends to
become trapped at the corners (1,1) or (1,0) each time it reaches them.
## 1
Although RBP considers the temporal dynamics of the strategy profile to
estimate how long a subtree remains prunable, it does not incorporate this
information into the pruning decision itself.
Authorized licensed use limited to: Rajshahi University Of Engineering and Technology. Downloaded on November 15,2026 at 09:38:45 UTC from IEEE Xplore.  Restrictions apply.

## 414IEEE TRANSACTIONS ON GAMES, VOL. 18, NO. 2, JUNE 2026
Fig. 3.  (Left) Phase plot of strategy evolution in the illustrative game example. To enhance visibility and prevent self-overlap along the trajectory, the plotted
trajectory is progressively rescaled, with the scaling factor increasing as the iteration count increases. The axes therefore do not represent the original probability
scale and are used only for visualization. Except for the starting point, the original trajectory lies entirely on the boundary of the[0,1]×[0,1]square. (Right)
Enlarged view of the upper-right region. The red segments show intervals where the trajectory is temporarily trapped at the corner and red annotations indicate the
corresponding iteration ranges of stagnation.
Take corner (1,1) as an example. At this point, the instantaneous
regret vector of player 2 is (0,0.001), which slightly favors player 2’s
second action. However, player 2’s accumulated regret for the second
action is negative, indicating that the second action is suboptimal
overall. As a result, the strategy profile remains unchanged for many
iterations, implying that updates to the strategy profile can be safely
skipped. As instantaneous regrets accumulate, player 2’s accumulated
regret for the second action eventually turns positive, prompting devi-
ation and naturally leading to resumed updates.
Formally, the reach probability for any game statehwithin a sub-
game can be factorized asπ
σ
## (h)=π
σ
## (h|α(h))π
σ
## (α(h)),whereα(h)
denotes the game state at the root of the subgame, withα(h)h.When
a subgame converges, the strategy profile and the utility of each game
state within the subgame remain fixed. Although the reach probability
π
σ
(α(h))of the subgame root may still vary across iterations, the
conditional componentπ
σ
(h|α(h))remains invariant. Consequently,
the reach probability and utility update step can be omitted for all
nonroot states within the subgame, since the corresponding values
can be reused from previous iterations. While updates to the average
strategy and regret are still required in the second and third steps of the
CFR iteration, they can be deferred until the subgame is reactivated.
Fig.2shows a schematic of SP.
Before applying SP to CFR, two key questions need to be consid-
ered: 1) under what conditions should a subgame be pruned and 2)
how to compensate for the skipped updates during pruning to ensure
consistency with CFR. To refine the pruning process, we propose two
mechanisms: pruning constraint checking and RM compensation.
## A. Pruning Constraint Checking
The constraints required to ensure an unchanged strategy profile in
the next iteration can be derived directly from(2).Letpbe a public
tree node, and denote the subtree rooted atpas Suc(p). If, for every
information setIin Suc(p), the positive regrets remain unchanged and
the negative regrets stay nonpositive, then the strategy profile within
Suc(p)will remain unchanged in the next iteration
∀a∈A(I),

r
σ
## T
i
(I,a)=0,ifR
## T−1
i
(I,a)>0
r
σ
## T
i
(I,a)≤−R
## T−1
i
(I,a),otherwise.
## (4)
AlthoughR
## T−1
i
(I,a)is unavailable during the pruning of a subtree,
the overall change in accumulated regrets during pruning fort∈[t
## 1
## +
## 1,t
## 2
]can be expressed as a sum of linear functions of the external
reach probability
## R
t
## 2
i
(I,a)−R
t
## 1
i
(I,a)
## =
## 
h∈I
## (u
σ
t
## 1
## (ha)−u
σ
t
## 1
## (h))π
σ
t
## 1
## −i
## (h|α(h))
t
## 2
## 
t=t
## 1
## +1
π
σ
t
## −i
## (α(h)).(5)
During pruning, the strategy profile inside the subtree remains fixed,
so for any historyhwithin the subtree, both(u
σ
t
i
## (ha)−u
σ
t
i
## (h))and
π
σ
t
## −i
(h|α(h))remain invariant and can thus be taken att
## 1
without loss
of generality. Therefore,R
## T−1
i
(I,a)can be efficiently recovered by
incrementally accumulating the external reach probabilities at the root
of the pruned subtree, rather than recomputing it through standard CFR
iterations.
The subtree continues to be pruned until the pruning constraints
are violated. Typically, verifying these constraints requires evaluating
each public node within the pruned subgame at every iteration to
determine whether pruning should proceed. To reduce the associated
computational cost, we introduce a more restrictive yet computationally
inexpensive sufficient condition for pruning. Specifically, if the strategy
profile within the subgame exhibits zero regret and the reach probability
at the subgame root remains unchanged, pruning can be maintained.
Under this condition, once the zero-regret condition is satisfied, it is
sufficient to verify the reach probability only at the root of the subgame,
rather than evaluating constraints in(4)at every public node. If this
stronger condition no longer holds, we revert to checking the original
pruning constraints. Moreover, small deviations from the zero-regret
condition and minor fluctuations in the root’s reach probability are
permitted. Although such relaxations may cause slight deviations from
the standard CFR dynamics, Lemma1shows that the accumulated
regret for any information set in the subgame increases by at most
one constant per iteration. Consequently, Theorem1ensures that the
average accumulated regret still converges at a rate ofO(1/
## √
## T),pro-
vided that the accumulated regrets of the information sets in the pruned
subgames remain bounded by a positive constant. This guarantees that
SP preserves the convergence properties of the CFR algorithm.
We refer to the pruning scheme that relies solely on the stronger
pruning condition as Partial Subgame Pruning (Partial SP), and the
one that reverts to the original pruning constraints once the stronger
condition is violated as Two-stage Subgame Pruning (Two-stage SP).
Lemma 1:For subgameSuc(p), suppose that the strategy profile in
Suc(p)had converged to aδ
## 1
-regret strategy when it was pruned since
stept
## 
## (i.e.,max
a
r
σ
t
## 
i
(I,a)≤δ
## 1
for allI∈Suc(p)andi∈N), and
Authorized licensed use limited to: Rajshahi University Of Engineering and Technology. Downloaded on November 15,2026 at 09:38:45 UTC from IEEE Xplore.  Restrictions apply.

## IEEE TRANSACTIONS ON GAMES, VOL. 18, NO. 2, JUNE 2026415
the change of the reach probability tophas remained smaller thanδ
## 2
## .
Then, for any information setIin the subgame, its regret grows no
more thanδ
## 1
+|I|δ
## 2
Δper iteration, where|I|denotes the number of
game states inI.
Proof:Let
## 
hdenote the vector of histories inI. By the assumptions
of the lemma, we have
max
a
r
σ
t
## 
i
(I,a)≤δ
## 1
∀I∈Suc(p)






π
σ
t
## 
i
## (h)−π
σ
t
i
## (h)






## ≤δ
## 2
## ∀h∈p.(6)
## Then
## 
h∈I

π
σ
t
## −i
## (h)−π
σ
t
## 
## −i
## (h)

u
σ
t
i
## (ha)−u
σ
t
i
## (h)

## ≤



π
σ
t
## −i
## (
## 
h)−π
σ
t
## 
## −i
## (
## 
h)



## 2
## ·



u
σ
t
i
## (
## 
ha)−u
σ
t
i
## (
## 
h)



## 2
## ≤|I|·



π
σ
t
## −i
## (
## 
h)−π
σ
t
## 
## −i
## (
## 
h)



## ∞
## ·



u
σ
t
i
## (
## 
ha)−u
σ
t
i
## (
## 
h)



## ∞
≤|I|δ
## 2
## Δ.(7)
Hence, the regret at iterationtis bounded by
r
σ
t
i
(I,a)=
## 
h∈I
π
σ
t
## −i
## (h)

u
σ
t
i
## (ha)−u
σ
t
i
## (h)

## ≤
## 
h∈I
π
σ
t
## 
## −i
## (h)

u
σ
t
i
## (ha)−u
σ
t
i
## (h)

+|I|δ
## 2
## Δ
## ≤δ
## 1
+|I|δ
## 2
## Δ.(8)
## 
Theorem 1:Suppose thatR
t
i
(I,a)≤(M−1)/2whenever infor-
mation setIis pruned. Then, after applying SP to CFR, the average
accumulated regret at any information set will converge to 0 at a rate
ofO(1/
## √
T)asTincreases.
## 2
Proof:We begin with the following expansion:


## [R
t
i
(I,a)]
## +


## 2
## 2
## =



## [R
t−1
i
(I,a)+r
σ
t
i
(I,a)]
## +



## 2
## 2
## ≤


## [R
t−1
i
(I,a)]
## +


## 2
## 2
## +



r
σ
t
i
(I,a)



## 2
## 2
## +2
## 
## [R
t−1
i
(I,a)]
## +
## ,r
σ
t
i
(I,a)
## 
## (9)
where·,·denotes the inner product, and[·]
## +
denotes the element-wise
positive-part operator.
On the one hand, whenIis pruned by SP at stept, Lemma1applies.
The norm of the instantaneous regret vectorr
σ
t
i
(I,a)is bounded by



r
σ
t
i
(I,a)



## 2
## ≤(δ
## 1
+|I|δ
## 2
## Δ)
## 
## |A(I)|.(10)
Given the assumptionR
t
i
(I,a)≤(M−1)/2for pruned information
sets, and further assuming thatδ
## 1
andδ
## 2
are sufficiently small such that
## (δ
## 1
+|I|δ
## 2
Δ)<1, we can bound the regret increment as follows:


## [R
t
i
(I,a)]
## +


## 2
## 2
## −


## [R
t−1
i
(I,a)]
## +


## 2
## 2
≤(M−1)(δ
## 1
+|I|δ
## 2
## Δ)
## 
|A(I)|+|A(I)|(δ
## 1
+|I|δ
## 2
## Δ)
## 2
≤M|A(I)|(δ
## 1
+|I|δ
## 2
## Δ).(11)
On the other hand, ifIis not pruned at stept, then by the properties
of RM,[R
t−1
i
(I,a)]
## +
## ,r
σ
t
i
(I,a)=0. This yields


## [R
t
i
(I,a)]
## +


## 2
## 2
## −


## [R
t−1
i
(I,a)]
## +


## 2
## 2
## ≤



r
σ
t
i
(I,a)



## 2
## 2
## ≤|A(I)|Δ
## 2
## .
## (12)
Since[R
## 0
i
(I,a)]
## +

## 2
## 2
=0, summing overTiterations gives


## [R
## T
i
(I,a)]
## +


## 2
## 2
## ≤T
## 
M|A(I)|(δ
## 1
+|I|δ
## 2
## Δ) +|A(I)|Δ
## 2
## 
## .(13)
## 2
An analogous result holds for CFR+ with SP.
Therefore, the accumulated regret at information setIis bounded
## R
## T
i
## (I)≤


## [R
## T
i
(I,a)]
## +


## ∞
## ≤


## [R
## T
i
(I,a)]
## +


## 2
## ≤

## 
M|A(I)|(δ
## 1
+|I|δ
## 2
## Δ) + Δ
## 
## |A(I)|

## √
## T(14)
which results in aO(1/
## √
T)convergence rate of the average accumu-
lated regret.
B. RM Compensation
## Afterpruningconcludes,theaccumulatedregretandaveragestrategy
within the subtree must be adjusted to account for the updates deferred
while pruning is active. Without this adjustment, CFR’s convergence
guarantees would no longer hold. Although incremental compensation
for the missed updates is straightforward, it incurs almost the same
computational cost as running standard CFR. Fortunately, all missed
updates can be efficiently corrected in one step.
The accumulated regret can be recovered using(5),asdiscussedin
the previous subsection. In addition, the weight of the current strategy
in the average strategy update is determined by a constant conditional
term and the sum of the player’s historical reach probabilities at the
root of the pruned subtree, which can be expressed as
t
## 2
## 
t=t
## 1
## +1
π
t
i
(I)=π
t
i
(I|α(I))
t
## 2
## 
t=t
## 1
## +1
π
t
i
(α(I)).(15)
Here, we reuse the notationα(I)to denote the ancestor ofIat the root
of the subgame and defineπ
t
i
(I)=π
t
i
(h)for anyh∈I, representing
playeri’s individual reach probability forI, which is well defined under
perfect recall.
Algorithm1outlines the CFR iteration with Two-stage SP. With
minor modifications, SP can be readily integrated into various CFR
variants, such as CFR+. In CFR+, accumulated regrets are constrained
to remain nonnegative, which eliminates the need for regret value
compensation—since compensation primarily corrects negative ac-
cumulated regrets. Adjustments to positive accumulated regrets are
infrequent and typically small; otherwise, the strategy would not have
stabilized. Consequently, omitting regret compensation has minimal
impact on the algorithm’s behavior. Overall, SP, as a pruning method
applied directly to public tree traversal, provides notable computational
benefits and can be easily adopted in CFR variants.
## V. EXPERIMENTS
We evaluate Partial SP and Two-stage SP on both CFR and CFR+
across several classic two-player zero-sum extensive-form games.
While both variants rely on the same pruning condition to trigger
pruning, they differ in their depruning criteria. For each setting, we
report the exploitability, pruning proportion,
## 3
and normalized wall-
clock time per iteration.
## 4
All wall-clock measurements are conducted
under identical hardware configurations to ensure fair and reliable
runtime comparisons. In contrast, exploitability and pruning proportion
are hardware-independent: given a fixed initialization, both CFR and
the associated pruning mechanisms are fully deterministic, yielding
reproducible results across runs. The full implementation is available
online.
## 5
## 3
Pruning Prop. (%)in TableII, representing the average fraction of public
nodes pruned per iteration
## 4
Avg. Time (norm.)in TableII, denoting the average wall-clock time per
iteration, normalized by the corresponding nonpruning (w/o SP) baseline.
## 5
[Online]. Available:https://github.com/67available/SubgamePruning
Authorized licensed use limited to: Rajshahi University Of Engineering and Technology. Downloaded on November 15,2026 at 09:38:45 UTC from IEEE Xplore.  Restrictions apply.

## 416IEEE TRANSACTIONS ON GAMES, VOL. 18, NO. 2, JUNE 2026
Algorithm 1:CFR with Two-stage SP.
1:Input:public treeP, iteration timesT
2:Output:public treeP
3:PrunedNodes←∅// pruned public nodes
## 4:t←0
5:whilet<Tdo
6:maxRegret←{}// node→max regret in subgame (default
## 0)
7:next←{}// node→next child index
8:S←[ ]  // init stack
## 9:  P
USHOP(P.root,False)
10:while Snot emptydo// Depth First Search
11:(p, compensate)←popS
## 12:    P
OPOP(p,compensate)
## 13:t←t+1
## 14:
15:ProcedureP
USHOP(p,compensate)
## 16:  R
EACHPROBUPDATE(p)
17:ifp∈PrunedNodesnotcompensatethen
## 18:    A
CCREACHPROBUPDATE(p)
## 19:ifnot R
EACHPROBSTABLE(p)then
20:fordinD
ESCENDANTOF(p)do
21:ifV
IOLATESPRUNECONSTRAINT(d)then
22:push(p, True)intoS;next[p]←0
## 23:break
## 24:else
25:    push(p, compensate)intoS
## 26:next[p]←0
## 27:
28:ProcedureP
OPOP(p,compensate)
29:if next[p]=|C
HILDREN(p)|then
## 30:ifcompensatethen
## 31:RM C
OMPENSATION(p)
32:PrunedNodes.remove(p)
## 33:else
## 34:U
TILITYUPDATE(p)
## 35:A
VERAGESTRATEGYUPDATE(p)
36:max_regret←RM(p)
37:maxRegret[p]←max
## 
max_regret,
## 38:max
c∈CHILDREN(p)
{maxRegret[c]}
## 
39:if maxRegret[p]=0then
40:PrunedNodes.add(p)
41:else// some children not yet considered
42:    push(p, compensate)back intoS
43:c←child ofpat indexnext[p]
## 44:next[p]←next[p]+1
## 45:    P
USHOP(c,compensate)
Two hyperparameters,δ
## 1
## =10
## −15
andδ
## 2
## =10
## −15
, control the tol-
erance for deviations from the zero-regret condition and fluctuations in
the root reach probability in(6), respectively. All experiments conser-
vatively set these values to ensure lossless convergence, although larger
values may yield more aggressive pruning without materially affecting
exploitability. Empirically, a smallerδ
## 1
plays a more critical role than
δ
## 2
in maintaining convergence accuracy.
We benchmark four OpenSpiel[12]environments—Leduc Poker,
Tiny Bridge 2P,Liar’s Dice,andTic–Tac–Toe—as well as a five-bet-
level variant ofLeduc Pokerreferred to asLeduc5[10]. All environ-
ments are two-player zero-sum imperfect information extensive-form
games, exceptTic–Tac–Toe, which is a perfect information game. These
games vary from 9.5 K to over 1.3 M states, covering a broad range of
gameplay dynamics.
We begin by examining the effect of SP on solution quality. As shown
in TableII, both variants maintain convergence accuracy comparable
to the baseline, with exploitability remaining nearly identical and
occasionally slightly lower due to iteration-level fluctuations in CFR.
Overall,theseresultsindicatethatSPhasanegligibleimpactonsolution
quality.
In the following sections, we first report results for Two-stage
SP, focusing primarily on pruning proportions and wall-clock time
improvements relative to the nonpruning baseline, followed by Partial
SP for comparison between different depruning criteria.
A. Two-Stage Subgame Pruning
Two-stage SP achieves the most pronounced improvements inTiny
Bridge 2PandTic–Tac–Toe, where a zero-regret strategy profile
emerges within a few iterations, allowing the entire game tree to be
pruned permanently. Significant reductions also occur inLiar’s Dice
andLeduc5: under CFR, only 3% and 5% of public nodes are visited on
average, while under CFR+, around 50% are visited. In contrast,Leduc
Pokershows a smaller effect, with 54% of public nodes pruned under
CFR and 4% under CFR+.
Across all environments, fewer nodes are pruned under CFR+ than
under CFR, a pattern that can be understood from two complementary
perspectives. From the perspective of the pruning criterion, the non-
negativity property of accumulated regrets in CFR+ makes it more
difficult to satisfy the pruning constraints. From the perspective of
algorithmic dynamics, CFR+ updates are more forgetful, as they clip
negative accumulated regrets at every iteration, preventing subgames
from remaining stable for long, thereby reducing opportunities for
persistentpruning.Furthermore,ingameswithlargerbranchingfactors.
such asLeduc5andLiar’s Dice, Two-stage SP tends to remove a greater
number of nodes. This may be attributed to the presence of many
subgames that are highly unfavorable to certain players, where the
strategy profile tends to stabilize rapidly or even become deterministic.
Meanwhile, in smaller or perfect information games, Two-stage SP can
sometimes identify a global zero-regret strategy profile early—a Nash
equilibrium—allowing the traversal to terminate entirely. In such cases,
it effectively functions as an early-stopping mechanism that markedly
improves computational efficiency.
Regarding runtime, Two-stage SP achieves dramatic speedups in
Tiny Bridge 2PandTic–Tac–Toe, where the average per-iteration wall-
clock time is reduced by more than three orders of magnitude. InLiar’s
DiceandLeduc5, the runtime improvements are modest, on the order
of 9%–15%. In contrast, inLeduc Poker, Two-stage SP does not yield
runtime gains and can even incur additional overhead. It is also worth
noting that a large pruning proportion does not necessarily yield a pro-
portional speedup. For instance, inLiar’s Diceunder CFR, Two-stage
SP prunes over 96% of public nodes, yet the observed wall-clock time
decreases by only approximately 15%. This discrepancy stems from
the nontrivial computational overhead associated with maintaining the
pruning mechanisms: performing pruning constraint checking or RM
compensation across entire subgames can incur costs comparable to
running standard CFR. When such operations are invoked frequently,
the additional overhead introduced by SP may offset much of its
potential runtime advantage.
B. Partial SP and Comparative Analysis
To further assess the effect of the pruning condition, we examine
Partial SP and compare it with Two-stage SP. This comparison allows
Authorized licensed use limited to: Rajshahi University Of Engineering and Technology. Downloaded on November 15,2026 at 09:38:45 UTC from IEEE Xplore.  Restrictions apply.

## IEEE TRANSACTIONS ON GAMES, VOL. 18, NO. 2, JUNE 2026417
## TABLE II
## CFR
## ANDCFR+ WITH/WITHOUTSP
us to estimate how many pruned nodes in Two-stage SP correspond to
truly zero-regret subgames versus those that only appear stable under
transient regret dynamics.
As shown in TableII,Tiny Bridge 2PandTic–Tac–Toeare entirely
pruned after only a few iterations in both variants, indicating con-
vergence to a global zero-regret strategy profile. For the remaining
games, the relaxed depruning rule in Two-stage SP allows substantially
more public nodes to be pruned, particularly under CFR. This suggests
that many of the subgames pruned by Two-stage SP are not strictly
zero-regret. Rather, they represent transient equilibria in which players
temporarily maintain their current strategies because previously ac-
cumulated regrets discourage deviation, even though the accumulated
regrets continue to evolve during pruning. Although Partial SP prunes
fewer nodes, it achieves a wall-clock runtime comparable to, or even
faster than, that of Two-stage SP, particularly inLiar’s DiceandLeduc5.
## Thiscanbeexplainedbythefactthatitonlyverifiesthestrongerpruning
condition specified in(6)at the root of each pruned subgame, whereas
Two-stage SP must re-evaluate(4)throughout the subgame whenever
the reach probability stability condition in(6)no longer holds. As a
result, Partial SP exhibits comparable or better runtime performance
than Two-stage SP, despite pruning fewer nodes.
## VI. CONCLUSION
This work has introduced SP, a novel pruning technique for CFR
that targets converged subgames. By freezing the strategies within
these subgames and deferring both RM and average strategy updates
until they are reactivated, SP reduces computational overhead while
preserving CFR’s theoretical convergence guarantees. Empirical results
have demonstrated that the method effectively improves efficiency by
pruningasubstantialportionofthepublictreeacrossvarioustwo-player
zero-sum extensive-form games. Future work may explore combining
SP with complementary techniques, such as Total RBP and EVP, and
reducing the computational overhead associated with maintaining the
pruning mechanisms themselves. In addition, while our experiments are
conducted on two-player zero-sum games, the proposed formulation
and mechanisms naturally extend to two-player general-sum games as
well as multiplayer settings.
## REFERENCES
[1] M. Johanson, K. Waugh, M. Bowling, and M. Zinkevich, “Accelerating
best response calculation in large extensive games,” inProc. 22nd Int. Joint
Conf. Artif. Intell., 2011, pp. 258–265.
[2] M. Zinkevich, M. Johanson, M. Bowling, and C. Piccione, “Regret mini-
mization in games with incomplete information,”Adv. Neural Inf. Process.
Syst., 2007, pp. 1729–1736.
[3] N. Brown and T. Sandholm, “Superhuman AI for heads-up no-limit poker:
Libratusbeatstopprofessionals,”Science,vol.359,no.6374,pp. 418–424,
## 2018.
[4] N. Brown and T. Sandholm, “Superhuman AI for multiplayer Poker,”
Science, vol. 365, no. 6456, pp. 885–890, 2019.
[5] M. Moravˇcık et al., “DeepStack: Expert-level artificial intelligence in
heads-up no-limit poker,”Science, vol. 356, no. 6337, pp. 508–513, 2017.
[6] O. Tammelin, “Solving large imperfect information games using CFR+,”
2014,arXiv:1407.5042.
[7] N. Brown and T. Sandholm, “Solving imperfect-information games via
discounted regret minimization,” inProc. AAAI Conf. Artif. Intell., 2019,
pp. 1829–1836.
[8] M. Lanctot, K. Waugh, M. Zinkevich, and M. Bowling, “Monte Carlo
sampling for regret minimization in extensive games,”Adv. Neural Inf.
Process. Syst., 2009, pp. 1078–1086.
[9] N. Brown and T. Sandholm, “Regret-based pruning in extensive-form
games,”Adv. Neural Inf. Process. Syst., 2015, pp. 1972–1980.
[10] N. Brown and T. Sandholm, “Reduced space and faster convergence
in imperfect-information games via pruning,” inProc. Int. Conf. Mach.
Learn., 2017, pp. 596–604.
[11] G. J. Gordon, “No-regret algorithms for online convex programs,”Adv.
Neural Inf. Process. Syst., 2006, pp. 489–496.
[12] M. Lanctot et al., “OpenSpiel: A framework for reinforcement learning in
games,” 2019,arXiv:1908.09453.
[13] Y. Shoham and K. Leyton-Brown,Multiagent Systems: Algorithmic,
Game-Theoretic, and Logical Foundations. Cambridge, U. K.: Cambridge
## Univ. Press, 2008.
[14] S.J.Brams,NegotiationGames:ApplyingGameTheorytoBargainingand
Arbitration. London, U.K., and New York, NY, USA: Routledge, 2003.
[15] M. Tambe,Security and Game Theory: Algorithms, Deployed Systems,
## Lessons Learned. Cambridge, U. K.: Cambridge Univ. Press, 2011.
[16] M. Bichler, M. Fichtl, S. Heidekrüger, N. Kohring, and P. Sutterer, “Learn-
ingequilibriainsymmetricauctiongamesusingartificialneuralnetworks,”
Nature Mach. Intell., vol. 3, no. 8, pp. 687–695, 2021.
[17] Y. Bai, C. Jin, S. Mei, and T. Yu, “Near-optimal learning of extensive-form
games with imperfect information,” inProc. Int. Conf. Mach. Learn., 2022,
pp. 1337–1382.
[18] B. Li and L. Huang, “Efficient online pruning and abstraction for imperfect
information extensive-form games,” inProc. 13th Int. Conf. Learn. Rep-
resentations, 2025. [Online]. Available:https://openreview.net/forum?id=
MTcgsz1SHr
Authorized licensed use limited to: Rajshahi University Of Engineering and Technology. Downloaded on November 15,2026 at 09:38:45 UTC from IEEE Xplore.  Restrictions apply.