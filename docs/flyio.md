# Flyio hosting

What commands were used (around november).

Get the fly cli tool, please use it responsibly (dangerous)

```bash
curl -L https://fly.io/install.sh | sh
```

Sign in, enter your credit card info, yadda yadda

```bash
fly auth signup
```

Configure your fly.toml

```bash
hx fly.toml
```

Create your dockerfile (see Dockerfile) and upload that dockerfile to flyio

```bash
fly launch
```

View your app in the browser to see the result.
