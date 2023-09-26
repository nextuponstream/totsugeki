# Totsugeki-display

Core library for displaying a bracket in a web page with Tailwind CSS. Tied to
`totsugeki` library as it takes output and reorders it to produce a new ordered
output for displaying.

Main responsibilties of this library are:

- partition matches per round
- give row hint to matches (to be used with Tailwind CSS dependency)
- drawing lines between rounds

Spacing is html page's responsibility (usually drawing things at equidistance
is enough). Matches are partitioned in rounds, then a row hint is given for a
given Tailwind CSS class to use. 

Current implementation allows to draw:

- winner bracket
- loser bracket
