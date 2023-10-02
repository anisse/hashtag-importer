# hashtag-importer

Import hashtags from other mastodon servers, into your instance — without being admin.

Build with:
```
cargo build --release
```

## But why ?

In the fediverse, your server might not see all posts made by everyone, for a variety of reasons. If your server is too small, you'll often have a global timeline and hashtags being un-usable. For server admins, a simple solution is to use relays, and Mastodon supports it. But what if you're not admin ? That's where this tool comes in.

It works by using the public hashtag timeline api of other servers, and then using the search api of your instance to import posts one by one.

## How to use

Beware: this isn't really set in stone yet. Installing the app is a two step-process:
```sh
hashtag-importer create-app # asks for your mastodon instances and registers the app there
# manually edit your config.toml auth section, adding client_id and client_secret
hashtag-importer user-auth # asks for permissions to your user account in order to read hashtag timelines and search for individual posts
# manually edit your config.toml auth section again, adding token
```

Then, you want to add hashtags to your `config.toml`. Have a look at the `config-sample.toml` examples to see how watched hashtags are configured.

The app is a long-running service. Run with:
```sh
hashtag-importer run
```
It expects the `config.toml` to be in the current directory.
