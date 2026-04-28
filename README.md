# wiki-any-percent

Program to speedrun a wikipedia speedrun.

## Step 1: Download Wikipedia SQL dumbs

Download from: `https://dumps.wikimedia.org/[lang]wiki/[date]`

You should look for [lang]wiki-[date]-[kind].sql.gz

We need these kinds: `pagelinks`, `linktarget`, `page`

Example: download latest englisch version

```bash
wget https://dumps.wikimedia.org/enwiki/latest/enwiki-latest-pagelinks.sql.gz
wget https://dumps.wikimedia.org/enwiki/latest/enwiki-latest-linktarget.sql.gz
wget https://dumps.wikimedia.org/enwiki/latest/enwiki-latest-page.sql.gz
```

Then extract:

```bash
gunzip ./enwiki-latest-pagelinks.sql.gz
gunzip ./enwiki-latest-linktarget.sql.gz
gunzip ./enwiki-latest-page.sql.gz
```

## Step 2: extract relevant information from SQL dumps

Suppose you have downloaded the SQL dumps into `./dumps` and created a `./extracted` directory

```bash
cargo r --release -- extract ./dumps/enwiki-latest-page.sql  ./dumps/enwiki-latest-pagelinks.sql  ./dumps/enwiki-latest-linktarget.sql  \
    ./extracted/enwiki-latest-pagenames.bin ./extracted/enwiki-latest-pagegraph.bin
```


## Step 3: Profit

Test by running:

```bash
cargo r --release -- speedrun any-percent-bidirectional ./extracted/enwiki-latest-pagenames.bin ./extracted/enwiki-latest-pagegraph.bin "42" "Hello, World!"
```

You can traverse the linked pages interactively with this:

```bash
cargo r --release -- list-linked-pages ./extracted/enwiki-latest-pagenames.bin ./extracted/enwiki-latest-pagegraph.bin
```
