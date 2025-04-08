# {
#   "for_testing": {
#     "value": {
#       "g1": [
#         "0x046c22da18069e0918ffba0ece48354d409675a2e820162cdd4abeabab5e951a",
#         "0x0e06cbbe2ca2177602f48201cba590a799f278d7e4cc8c850c1e4d091e976a21"
#       ],
#       "g2": [
#         [
#           "0x0872beb879459a97449dd630a3531bdc1a2ec5da8c0c1ea688a4200977920505",
#           "0x29a886376e351d67b8635d770fb9754d62aeb1950110ce96a98e9e53ec7373ef"
#         ],
#         [
#           "0x2bfa8b34caa85f41191682d5a63a0f265b90e40f5ce39b67ba9161609df09637",
#           "0x20fe3f8444a8f134bb1276e083dbfdac5af60cb5303b9aa21e32031f580bf2f5"
#         ]
#       ],
#       "priv_key": "0xffe3be6f94645e9216938adbaa5e621cd4afd69ffd75fb433498ca18866b248c",
#       "call_pubkey_registration_message_hash_result": "2c1619993b1ae6dcb33661d64742b2b7336a90c3db7dfaba6eb691d98fea060a0a16f975b962fecbe821b85c2d96093a9db1f2cf12b878a2376d99a16c4d9f06",
#       "sig_out": "180da242dad6993b00cc7600f46bda6aa5e7c4987d5056f7581e1c7596aa930109f05112657683c71e990e039c84af4b41cfc2b2266e98c23ca0e83fa7b81b78"
#     }
#   }
# }
#
# cast call to sig "(g1.X,g1.Y)" priv_key
cast call 0x5FbDB2315678afecb367f032d93F642f64180aa3 \
  "scalar_mul((uint256,uint256),uint256)((uint256,uint256))" \
  "(0x046c22da18069e0918ffba0ece48354d409675a2e820162cdd4abeabab5e951a, 0x0e06cbbe2ca2177602f48201cba590a799f278d7e4cc8c850c1e4d091e976a21)" \
  0xffe3be6f94645e9216938adbaa5e621cd4afd69ffd75fb433498ca18866b248c


