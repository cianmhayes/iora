# Iora is the Irish word for squirrel🐿️

Iora is an end to end static asset hosting and caching system written in rust. It is designed as a set of layered caches, some of which may be remote. The caller requests an asset based on a name and a version pattern, and iora takes care of managing the cache at each layer and calling through as necessary.

Iora is my first rust project of any real complexity and my intent is that over time the rust will become more idiomatic, and any strange patterns or technical choices I've made will be fixed. It's primary motivation is as a learning project.
