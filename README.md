# hashtag-importer

Import hashtags from other mastodon servers, into your instance — without being admin.

Build with:
```
cargo build --release
```

## But why ?

In the fediverse, your server might not see all posts made by everyone, for a variety of reasons. If your server is too small, you'll often have a global timeline and hashtags being un-usable. For server admins, a simple solution is to use relays, and Mastodon supports it. But what if you're not admin ? That's where this tool comes in.

It works by using the public hashtag timeline api of other servers, and then using the search api of your instance to import posts one by one.

Have a look at the `config-sample.toml` file for how watched hashtags are configured.


## How to use

TODO
