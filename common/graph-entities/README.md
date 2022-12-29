# graph-entities

This lib allow you to modelize a `Response` in a form of a `Graph`, and give the
ability of this `Graph` to be formatted.

For instance, we could have a `Graph` like this

```
  ┌──────────┐  fieldA ┌────────┐
  │Container ├─┬──────►│ Node A │
  └──────────┘ │       └────────┘
               │
               │
               │ foo   ┌────────┐    0   ┌──────┐
               └──────►│ List   ├─┬─────►│Node B│
                       └────────┘ │      └──────┘
                                  │
                                  │  1   ┌──────┐
                                  ├─────►│Node C│
                                  │      └──────┘
                                  │
                                  │  2   ┌──────┐
                                  └─────►│Node D│
                                         └──────┘
```

Which would be formatted in:

```json
{
  "container": {
    "fieldA": { ...NodeA }
    "foo": [{ ...NodeB }, { ...NodeC }, { ...NodeD }],
  }
}
```

We did store everything as:

```
data: Hashmap<Id, Node>
root: Node
```

But with this approach, it means, we can't have selection splitted for each
Node. We'll need to iterate it.

A possible solution would be to change the Id format, every node got an Internal
ID which is used to link node together, but some can have a `NodeId`, and we can
update Nodes by `NodeId`.

-> Not working: we do not have a proper selection field right now, so when there
is an update for a cyclic node for instance, it'll be hard to figure out what is
needed

## Long plan

The long plan is to have the SelectionStep which should be used inside the
GraphQL flow, the ExecutionStep will fetch the data needed, then the
SelectionStep will be the translation of the GraphQL Request to use the fetched
Data and create a Graph.

Why do we want to split those two behavior completly?

It'll be better to manage everything related to live queries, we'll be able to
add the new data to the Data layer, run the selection step again, and discard
the unused data from the selectionStep.

## Cyclic References

What happens when we do want to have `Node A` multiple time, with not the same
data showed? We need to implement a selection mecanism which would be decoupled
from the `data node`.

Also this selection mecanism needs to be `Serializable` if we want to share it.

We can imagine a `struct Selection` which would have this API:
