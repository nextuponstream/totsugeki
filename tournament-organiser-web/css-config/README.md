# Bundling css for displaying brackets

## Context

Currently, the way to display brackets for a dynamic number of participants is to bundle all required css for it. Tailwind compiles the following classes:

* `row-start-N`
* `grid-cols-N`

While also needing to compile `gridTemplateColumns` for all possible values.

Because values are dynamic, they get purged while bundling any css.

The user should not have to request css beyond reasonable values. But what is reasonable?

* 7k people for Street Fighter 6 at Evo Vegas in 2023, divided in pools for logistics reason
* 100 people for online bracket (not divided in pools)
* small to medium tournaments rarely exceed 50 people, usually divided in 2-3 pools

## Current line of thinking

While one could dvelve on what is right, I'll go ahead and propose this:

Bundling the css for 40k people tournament should be futur proof enough. Double current maximum expectations (10k), then double it again (so x4). If someone encounter a problem with the size of the css, well I guess that's another problem that is too early to think about. Right now, it's more important to be able to display most realistic brackets.

It is and will remain a limitation that you must ship some css with this current implementation. Maybe that will be a problem to solve in the future but we are not there yet.

## Generate css

* TODO handle css compilation with a separate command
* TODO extract css file generation in separate library: totsugeki-ui
* TODO extract binaries for css generation in totsugeki-native-app into totsugeki-ui