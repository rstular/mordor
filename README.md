# README

Mordor is in development and should be treated with caution. Talk with Rok or Robert before using it (anywhere). At the moment, it should only be used when access for external users is needed to a website that also has TU Delft SAML login (i.e., NetID access) on these webservers:
- `mude.citg.tudelft.nl`
- `interactivetextbooks.citg.tudelft.nl` (not yet configured)

Both of these webservers were provided SAML credentials in summer of 2023 (Robert submitted the ICT tockets via TopDesk). They are valid for 10 years.

Please consult with Rok before deploying mordor anywhere else; there is a certain configuration that it is technically capable of supporting, but shouldn't be doing it in order to be compliant with ICT directives: it must not be used as and IDP proxy, meaning that it is not allowed to use SAML credentials of one domain to authenticate into another. In other words, it is forbidden to use `mude.citg.tudelft.nl` SAML credentials to perform authentication for a different domain.

## External User Login

There's a script on the server that can be used to give external users access to the website. You can run it with the following command:
```
sudo /srv/utilities/add_user.py "desired_username"
```
You'll then be prompted to enter a password, which will then in turn be added to the database.

Another script is present to dump logs from the login service:
```
/srv/utilities/dump_logs.py
```
