Changes for Safety
------------------

`CEdgeStore` is replaced by `typed_arena_nomut::Arena<CEdge>`. 

`CEdgeStore` is an arena with built-in stack storage for the first allocation
of the arena. It exposes the allocated buffers to support very fast allocation,
and supports fast enumeration by returning pointers to each allocation.

`CCoverageBuffer` also now uses a `typed_arena_nomut::Arena<CEdge>` but uses it
to allocate `CCoverageIntervalBuffer`'s. We currently lack support for
the builtin stack storage. Storing these in an Arena is not ideal, we'd rather
just heap allocate them individually.



