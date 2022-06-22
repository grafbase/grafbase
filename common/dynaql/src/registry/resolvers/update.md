# How we will store relation between entities?

Right now, relation are stored based on the entity type.
We'll store them based on the relation name if there is a relation name, if there
isn't, we'll infer a new relation name within the parser?

When we have a relation defined as `published` for instance between a POST and a USER

From the User we'll want to fetch the Associated Post
From the Post we'll want to fetch the Associated User

We could imagine changing the GSI3 pk to the relation name and SK to the `__pk` to allow
grouping Node & Edges together

This GSI3 will allow us to optimize fetch relations in the future.

A new attribute will be necessary inside the relation: `__relation_name`


## Parser

From the Parser side, we'll need to generate the relations based on our relation engine

## Update

For the Update the GSI give us an inversed index, so we'll use it to update values.
