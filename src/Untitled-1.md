GET /<id> forwarder til en databased URL.
GET /<id>/qr?size=300 tegner en QR-kode til /<id>
GET /<id>/info returnerer et JSON-objekt med metadata
POST / tager et JSON-objekt {"url": "..."} og returnerer et JSON-objekt med metadata

Metadataen indeholder stored_id, stored_url

Som noget sjovt kan du gøre, så GET /<id>/qr?size=300 kigger på om der bliver efterspurgt text/plain eller image/png og enten bruge crate'en qrcode til at tegne en ASCII qr-kode, eller et billede.
