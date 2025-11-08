

# Gossip 
- Today, guests subscribe to a single global gossip topic. However, there is the ability to "fake" topics while publishing messages 

```json
// Inbound / Outbound Messages
{
  "topic" : "my-cool-topic",
  "content" : {
    "foo" : "bar"
  }
}
```

# Todo
- Replace KV tempfile with actual persistance..


Guests can be generated from a template using XTP
```
xtp plugin init --schema-file <path/to/fern-schema.yaml>
```