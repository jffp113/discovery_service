# Service Discovery

I'm doing this project just to learn rust



## Extract a dns sample request

```bash
nc -u -l 1053 > query_packet.txt
dig +retry=0 -p 1053 @127.0.0.1 +noedns google.com
```