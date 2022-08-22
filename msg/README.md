## msg

Generates constants for the message offsets for used languages. Run ``` pyhton xx/gen_xx.py ``` to update message constants for language xx. 

### Message format specification

Each message is encoded in binary format. Its components are:

* *ID*: uint16 - message ID 
* *Type*: uint16 - places message into a broader category
* *Command*: uint16 - describes the general purpose of the message.
* *Key*: uint16 - specifies the payload - either the type of the SDR (e.g. level, encoding)  
* *Payload*: depends on the message key - can be a parameter value (float) or a SDR (bit array)


```
-----------------------------------------------------------------------------------------------------------------
||                            Header                            ||                     Body                    ||
-----------------------------------------------------------------------------------------------------------------
||      ID      |     Type      |    Command    |      Key      ||                    Payload                  ||
-----------------------------------------------------------------------------------------------------------------
||      0-1     |      2-3      |      4-5      |      6-7      ||                     8-n                     ||
-----------------------------------------------------------------------------------------------------------------
