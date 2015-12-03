pub static CODE: &'static str = r###"'home/kneil mkdir
'root mkdir
[fread prln]'bin/cat fwrite
[ls 'prln map]'bin/ls fwrite
[len =len 0 =i [dup "i dup lsh3 swap nth 13 + prchr "i 1 + dup =i "len eq]while] 'bin/rot13 fwrite
[access /home/kneil, password kl3pto
escalate privileges to kill root's vscan process
how? that's your job
i'll handle the rest once you're out

regards,
	atage]'mnt/jack/readme fwrite
[Welcome to /forth. We try to keep things a little more civil around here than /lisp. Please keep language debates to that directory
Forth is a dynamically typed stack machine. Instructions are parameterless, & thus the language exists as a series of whitespace separated tokens
A string in Forth is enclosed in square brackets: [like so]. This allows nesting, [as in [this case]]. Specific bytes may be specified through \\ escapes: [\\0a] for a newline. Additionally unicode may be specified with \\uXXXX
The ease of nesting is important in Forth. Forth has no internal representation; a forth string is equivalent to raw machine code. Some may accuse me of ideological design, but 
~hefo
]'shared/forth/manual
[Could someone head downstairs & educate those hardhats on what they're making?
st should be liberated from its misnomer upbringing. Rather than viewing it as a data stack it should be viewed as a code buffer. ie code _is_ data (NOT strings!)
The syscall interface should be passing slices of the code buffer, not the whole kitchen sink. Right now if one wants to securely withhold data from children callbacks,
they must '=a cb [0 =a .]'
~psst

Your Lisp programs tend to use variables to store data since they lack the utility of the stack. You'd still have to shadow those variables, as the root issue is dynamic scoping. Really, why aren't we leaving name resolution to a compilation phase?
~iowart

Has anyone compared the performances of the shells? These implementations could really use an internal representation so that we might simplify the underlying architecture
~aawal

The Lisp crew _has_ implemented an internal representation. Wipe the mud from your brow & recognize that programs are trees, will always be trees, & should never not be trees
Stop trying to flatten our world perspective. My code should be able to introspect itself & others without having to deserialize your mashed potato representation
~psst

Internal representation? The architecture cleanly separates code from data. I say data loosely; this is clearly an object oriented architecture. What'd be nice is if the internal machine state was itself represented within the object data model. Speaking from an abstract machine standpoint, syscalls should exist to query machine state & replace it. Runtimes should be responsible for handling multiple machine states as contexts to deal with hiding information from inner contexts. These complaints aren't about our intermediate representations; we each have our own control on that front. It's about each wanting to mandate the abstract machine state's format to carry their desired runtime state. Were the runtime state as flexible as a single slot of our forth stack, this debate could be thrown under the rug & instead we could let our internal political squabbles manifest in uncooperative runtimes rather than a design by committee architecture (where then the grown ups who will cooperate still pay the price of all y'all)
~iowart

The machine architecture's state should include an intermediate representation. Hasn't anybody looked into the cost of all these specialized chips? There's a lot of duplicated effort being exercised to implement cheap lisps when right down the circuit board is an effort to implement a real lisp
~psst

In reality your cheap attempt at a real lisp is being optimized by the chip manufacturing to use the forth chip as an ALU. You should stop by downstairs for some coffee now & then, get an education in what we're really making
~aawal

You're only able to say that because the commonalities are implemented on your chip turf due to political history
~psst
]'shared/forth/plslisp fwrite
[There was some discussion in having a lisp implementation compile to Forth, but those discussions were by those who didn't understand that Lisp isn't backwards forth
(+ 1 2 3) is _not_ 1 2 + 3 +
The internal architecture of Lisp's implementation only relies on Forth insofar as one requires a machine to execute it

Now then, before I get into another tangent on how we should've made a Lisp machine, Lisp Semantics:

Programs are trees. A tree may contain atoms. Atom types are integers, strings, lists, or the singleton ə. We write a tree as (a b (12 3))
When one passes such a tree to the Lisp machine it iterates over each list recursively, evaluating builtins if the first element of the list is a string matching the name of a builtin
Here I will note 2 special cases:
The compiler, which parses the textual tree into an actual tree, specially interprets (#prefix * b (b @)) so that a then on (f *asdf q) would become (f b (b asdf) q)
The prelude uses this mechanism to implement 4 prefixes: " ' $ !

The other is that the runtime specially treats the builtin " to _not_ evaluate further. Therefore (" a (+ 1 2)) results in (" a (+ 1 2)), not (" a 3). We refer to this as deepquote

Expansions may result in multiple terms, or none at all

While one might find thinking in terms of symbolic substitution, they would be misguided. An expression that results in another executable expression will not execute
Thus if I store + in a, (($ a) 1 2) does not result in 3, it results in (+ 1 2). One would have to write ((($ a) 1 2))

Now then, nasty edgecases aside,
BUILTINS

+	Sum all paramters
-	Subtract from first parameter remaining parameters
*	Multiply
/	Divide from first
%	Remainder of divide
&	Bitwise-&
|	Bitwise-|
^	Bitwise-^
if	Based on first, return second or remaining after second
cmp		(a>b)-(b<a)
concat	(concat (a b c) 3 (4 5 6)) -> a b c 3 4 5 6
tail	(tail (a b c) 3 (4 5 6)) -> (b c) 3 (5 6)
print	Print all parameters
prchr	Print integers as characters
getline	Return a line from stdio
len		(len (a b c) 3 (4 5 6)) -> 3 -1 3
nth		(len 2 (a b c) 1 (a b c) 3 (a b c)) -> c b ə
slice	(slice 2 4 a b c d e f) -> c d
=	Assign string of first: remaining to local scope
$	Expand from first parameter
~	Assign string of first into nearest scope which already contains first, otherwise assign at top level, remaining
'	(' a (+ 1 2)) -> (a 3)
"	(" a (+ 1 2)) -> (" a (+ 1 2))
inline	(inline a (+ 1 2)) -> a 3
qlen	Result is the number of parameters qlen received
typeof	Replace each atom with its type integer
!		Here we come across the wrinkle of Lisp's simplicity. This is the evaluate-function-tree-with-scope builtin
	(! (a ..b c (tail (" + $a $c (- $b)))) 2 12 8 3) -> (+ 2 3 (- 12 8)) -> 9
	nb a new scope is created, with the variables a b c having consumed the remaining paramters of !

BRACKETS
We use all 4 brace types, () {} [] <>
() is the plain list constructor
{a b c} -> (' a b c)
<a b c> -> (" a b c)
[] is used for parsing strings, much as you would in Forth (nb, however, that we do not waste time encoding our programs with strings past compilation!)

PREFIXES
" "a -> (" a)
' 'a -> (' a)
$ $a -> ($ a)
! !a -> ! ($ a)

PRELUDE
Prelude should be executed before executing any other code in Lisp, it defines the following:
fn: (!fn name ..parameters <body>) creates a function which can be executed with ! (The astute reader will realize that fn must then not be defined through fn)
eval1: (eval1 {+ 1 2}) -> 3
eval: (eval + 1 2) -> 3
And of course: neg, prln, not, boo, eq, neq, gt, lt, gte, lte
~psst
]'shared/lisp/semantics fwrite
[How does anyone keep which brackets they should be using straight in Lisp? It seems like half the time the program executes what I don't want it to, & the other half of the time it refuses to execute that which I do want it to
~aawal

It isn't so bad. It can be a little hard keeping track of which calls evaluate to lists & which evaluate to expansions. But similar needs for documentation exist with Forth's necessity that the caller know the stack consumption behavior of the caller
~iowart
]'shared/lisp/semanticslol fwrite
[Someone put a leash on those Lispers, their programs are getting exceedingly obnoxious. You're not evolving the field of AI by writing a dictionary attack. ******* script kiddies
~aawal

This is a system generated response. Please moderate your language. Your message has been rectified.
~lord

*** ***
***     ***     *** *** ***  ***   ***
*** *** ***     *** ***      *** ***
***     *** *** *** *** ***  ***   *** these bots. Can't even learn to recognize ascii art. The apple doesn't fall far from the tree
~aawal

This is a system generated response. Please moderate your language. Your message has been rectified.
~lord]'shared/apocalypse
[login: ]print
'atage prln
[password: ]println
'guest ' 'atage mkuser
' 'atage login"###;