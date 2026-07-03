# br1ef

Your morning digest — email, right in your terminal.

Reads the last 7 days of email from your inbox and prints **From**, **Subject**, and **Body** to stdout. No GUI, no daemon, no sync. Just your email when you ask for it.

## How it works

br1ef connects to your email server via **IMAP** — the universal protocol for reading email. Every provider supports it: Gmail, Outlook, iCloud, Fastmail, even custom domains. No API keys, no OAuth, no provider-specific SDK.

When you run `br1ef`, it:

1. Loads your IMAP credentials from `.env`
2. Connects to the server over TLS
3. Searches your inbox for messages from the last 7 days
4. Extracts the plain-text body (HTML emails are converted to text)
5. Prints them to your terminal

## Setup

### 1. Create your `.env`

```bash
cp .env.example .env
```

### 2. Fill in your credentials

```
IMAP_HOST=imap.gmail.com
IMAP_PORT=993
IMAP_USERNAME=you@gmail.com
IMAP_PASSWORD=your-app-password
```

Replace with your provider's IMAP server. Port 993 is standard for IMAP over TLS.

### 3. Get an app password

Most providers **do not** let you use your regular password for IMAP. You need an **app password**:

- **Gmail** — enable 2FA, then visit [myaccount.google.com/apppasswords](https://myaccount.google.com/apppasswords)
- **Outlook** — enable 2FA, then visit [account.live.com/proofs/AppPassword](https://account.live.com/proofs/AppPassword)
- **Fastmail / others** — use your regular password (check your provider's docs)

## Run

```bash
cargo run
```

If everything is set up, you'll see:

```
───
From:    Alice <alice@example.com>
Subject: Lunch today?

Want to grab lunch at 12:30?
───
From:    Bob <bob@example.com>
Subject: Re: project update

Looks good. One comment inline...
───
3 email(s) in the last week.
```

## Why IMAP?

Because it works everywhere. Instead of writing a separate source for Gmail API, Outlook Graph API, and every other provider, br1ef speaks IMAP — one protocol that every email server already supports. If you can add your email to a phone or desktop mail client, you can use br1ef.

## Status

Version 0.1. Works with any IMAP mailbox. More sources and features coming.
