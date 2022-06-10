0.1.165 - 2021-11-04
===================

### Features
* Add twitter forwarder by @yzernik in https://github.com/yzernik/squeaknode/pull/1755
* Fix button text for submit button in add twitter account dialog by @yzernik in https://github.com/yzernik/squeaknode/pull/1757
* Make rpc enabled config set to false by default by @yzernik in https://github.com/yzernik/squeaknode/pull/1758
* Add custom config for autoconnect peer interval by @yzernik in https://github.com/yzernik/squeaknode/pull/1759

0.1.164 - 2021-11-02
===================

### Features
* Include network name in profile dropdown in header by @yzernik in https://github.com/yzernik/squeaknode/pull/1743
* Add license section to readme by @yzernik in https://github.com/yzernik/squeaknode/pull/1747
* Make rpc enabled by default by @yzernik in https://github.com/yzernik/squeaknode/pull/1749
* Update received offers in display even when none downloaded by @yzernik in https://github.com/yzernik/squeaknode/pull/1750

0.1.163 - 2021-10-28
===================

### Features
* Make download async by @yzernik in https://github.com/yzernik/squeaknode/pull/1725
* Add download in progress dialog @yzernik in https://github.com/yzernik/squeaknode/pull/1732
* Remove hash from title of buy squeak dialog by @yzernik in https://github.com/yzernik/squeaknode/pull/1741

0.1.162 - 2021-10-24
===================

### Features
* Add tests for db queries
* Make received offer retention a config by @yzernik in https://github.com/yzernik/squeaknode/pull/1710
* Move external address config to server section by @yzernik in https://github.com/yzernik/squeaknode/pull/1712
* Rename configs in server section by @yzernik in https://github.com/yzernik/squeaknode/pull/1713
* Update configuration doc for server configs by @yzernik in https://github.com/yzernik/squeaknode/pull/1714
* Rename rpc config section by @yzernik in https://github.com/yzernik/squeaknode/pull/1716
* Make update subscriptions async by @yzernik in https://github.com/yzernik/squeaknode/pull/1723

0.1.161 - 2021-10-14
===================

### Features
* Add more tests
  - by @yzernik in https://github.com/yzernik/squeaknode/pull/1525
  - by @yzernik in https://github.com/yzernik/squeaknode/pull/1526
  - by @yzernik in https://github.com/yzernik/squeaknode/pull/1527
  - by @yzernik in https://github.com/yzernik/squeaknode/pull/1530
  - by @yzernik in https://github.com/yzernik/squeaknode/pull/1533
  - by @yzernik in https://github.com/yzernik/squeaknode/pull/1541
  - by @yzernik in https://github.com/yzernik/squeaknode/pull/1543
  - by @yzernik in https://github.com/yzernik/squeaknode/pull/1548
  - by @yzernik in https://github.com/yzernik/squeaknode/pull/1560
  - by @yzernik in https://github.com/yzernik/squeaknode/pull/1562
  - by @yzernik in https://github.com/yzernik/squeaknode/pull/1564
  - by @yzernik in https://github.com/yzernik/squeaknode/pull/1578
  - by @yzernik in https://github.com/yzernik/squeaknode/pull/1579
  - by @yzernik in https://github.com/yzernik/squeaknode/pull/1580
  - by @yzernik in https://github.com/yzernik/squeaknode/pull/1583
  -  by @yzernik in https://github.com/yzernik/squeaknode/pull/1586
  - by @yzernik in https://github.com/yzernik/squeaknode/pull/1587
  - by @yzernik in https://github.com/yzernik/squeaknode/pull/1588
  - by @yzernik in https://github.com/yzernik/squeaknode/pull/1590
  - by @yzernik in https://github.com/yzernik/squeaknode/pull/1591
* Use deserialized block header in info object by @yzernik in https://github.com/yzernik/squeaknode/pull/1536
* Use decorator for protobuf flask routes by @yzernik in https://github.com/yzernik/squeaknode/pull/1538
* Create lightning client base class by @yzernik in https://github.com/yzernik/squeaknode/pull/1542
* Add toggle for connect with tor by @yzernik in https://github.com/yzernik/squeaknode/pull/1550
* Add rpc to get default peer port by @yzernik in https://github.com/yzernik/squeaknode/pull/1556
* Rename core crypto module by @yzernik in https://github.com/yzernik/squeaknode/pull/1563
* Remove secret key field of sent offer by @yzernik in https://github.com/yzernik/squeaknode/pull/1568
* Add check for payment point in unpack offer by @yzernik in https://github.com/yzernik/squeaknode/pull/1577

0.1.160 - 2021-10-07
===================

### Features
- Add unit test for timeline query.
    - #1518
- Add OpenTimestamps github action.
    - #1524

0.1.159 - 2021-10-04
===================

### Features
- Improve handling of duplicate entries in database.
    - #1478
- Refactor core functions to separate modules
    - #1460, #1464, #1468, #1473

0.1.158 - 2021-10-01
===================

### Features
- Show full date and time of squeak in tooltip.
    - #1393 PR by @abhiShandy

0.1.157 - 2021-09-28
===================

### Fixes
- Upgrade SQLAlchemy to version 1.4.
    - #1438

0.1.156 - 2021-09-27
===================

### Fixes
- Fix display external address text with full width.
    - #1429 PR by @abhiShandy

0.1.155 - 2021-09-27
===================

### Features
- Make p2p protocol more efficient by only requesting
  incremental changes when interests are updated.
    - #1424 and #1423

0.1.154 - 2021-09-24
===================

### Features
- Include saved peer name in list of connected peers.
    - #1361 and #1362
- Add button in UI to convert connected peer to saved peer.
    - #1365
- Improve dialog for showing external address, and include copy to
  clipboard.
    - #1380 PR by @abhiShandy
- Show connection status next to each item in saved peers list.
    - #1399

### Fixes
- Fix bug in p2p connection where the connection does not shut down
  properly in some cases.
    - #1397
- Fail gracefully in peer connection when LND is not available.
    - #1389 and #1395

0.1.153 - 2021-09-19
===================

### Features
- Better p2p networking. Inactive peer connection is now detected with
  `ping` message and stopped.
    - #1342 and #1347
- Better peer connection UI. `ConnectPeer` RPC is now synchronous, and
  the connect peer dialog in the frontend now updates with the result
  of the RPC, or error message.
    - #1326, #1327 and #1332
- Full-text search on squeak content.
	- #1287 and #1293
- Change display unit from msats to sats in received payments and sent
  payments.
	- #1341
- Show waiting indicator for more pages and dialogs in frontend UI.
	- #1338, 1343, 1344

### Fixes
- Fix broken unit test caused by unpinned dependencies in requirements.txt
    - #1310
