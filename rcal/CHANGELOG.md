# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [3.0.1] - 2026-10-28
### Bug Fixes

- Gate unused super::* import under cfg(feature = "zmq")([`8727992`](https://github.com/frA-GRA/rcal/commit/87279922490e3b5fad56e331f1bc4e8178c6f8de))
- **example**: Switch system_status example to DDS transport([`ea5f194`](https://github.com/frA-GRA/rcal/commit/ea5f194261e7dc1dae9c4498033672afd137f155))
- **exi**: Remove pretty option from Exi externalizer config([`51e5ef4`](https://github.com/frA-GRA/rcal/commit/51e5ef45d217ff0db1d06d886d492f7d611eca4b))
- **activemq**: Connect reader transport eagerly to fix hung second connection on Windows([`1ce6119`](https://github.com/frA-GRA/rcal/commit/1ce611959cc92af574b128cd42b848ba598199b4))
- **activemq**: Prefer IPv4 when resolving broker address([`174bea3`](https://github.com/frA-GRA/rcal/commit/174bea350a1cb5113c30d7dc995f580e23a25cd9))
- **rcal**: Remove unused test function([`0e27a09`](https://github.com/frA-GRA/rcal/commit/0e27a09a08586edbbcdb0e26ba6859253ab74986))
- **rcal**: Change hard-coded pi value to the rust constant([`e25a30c`](https://github.com/frA-GRA/rcal/commit/e25a30c12631df47fd714924cde88422260aa368))
- **tests**: Start ActiveMQ container only once per test binary([`0244593`](https://github.com/frA-GRA/rcal/commit/0244593a42e21e5626ccfdcab28ad4fecf860006))
- **cal_bridge**: Remove MessageTransformer. It's not used and design of the interface shouldn't be done yet.([`99bb578`](https://github.com/frA-GRA/rcal/commit/99bb57837dee0bd984418da2020126486cca750a))
### Features

- **asb**: Make DDS default transport, ZMQ optional ([#107](https://github.com/frA-GRA/rcal/pull/107))([`3c3f811`](https://github.com/frA-GRA/rcal/commit/3c3f811a1e0591b26b27148a56f504f79e298bce))
- **adsb_sim**: Add Dockerfile and fix asb_dump feature gate([`e16f6f9`](https://github.com/frA-GRA/rcal/commit/e16f6f9f97552767402094223235e632617e7b23))
- **asb**: Add ActiveMQ/STOMP AbstractServiceBus implementation ([#31](https://github.com/frA-GRA/rcal/pull/31))([`b81d5bb`](https://github.com/frA-GRA/rcal/commit/b81d5bb7201c2aecbfd66ed781d9e5d6d2769302))
- **exi**: Add EXI externalizer with schema-less byte-aligned codec ([#110](https://github.com/frA-GRA/rcal/pull/110))([`4fa84ad`](https://github.com/frA-GRA/rcal/commit/4fa84ad700ddfe00d7be62e5964a162a1ea491e1))
- **bench**: Add criterion benchmarks for EXI codec, serialization, and UUID([`36a9daa`](https://github.com/frA-GRA/rcal/commit/36a9daad87b71a3378fb754eeaee3dd65eb139a2))
- **bench**: Add compression externalizer benchmarks([`6bff1e3`](https://github.com/frA-GRA/rcal/commit/6bff1e319f41dab5e885aab74eacedc0014903a7))
- **tests**: Auto-start ActiveMQ broker via docker/podman([`0a55098`](https://github.com/frA-GRA/rcal/commit/0a55098575f491f4867ed0c5d5a777c8513ea3a0))
### Refactoring

- **dds**: Ponytail and caveman review cleanups([`a8388fb`](https://github.com/frA-GRA/rcal/commit/a8388fb2be81b5f72cadb5bac47e5507fe79d0ff))


## [3.0.0] - 2026-09-16
### Bug Fixes

- Various correctness fixes across adsb_sim and rcal([`1b22457`](https://github.com/frA-GRA/rcal/commit/1b2245716ae52c1d1f82a3ceb5a231a19f7374e9))
- **asb_view**: Restore asb_dump topics and fix ZMQ slow-joiner in adsb_sim([`f371ee8`](https://github.com/frA-GRA/rcal/commit/f371ee8b6686ed1aca6b0098b4d4da8cb28fcad6))
### Features

- **calconfig**: Add extensions field for service-specific config([`2eb3442`](https://github.com/frA-GRA/rcal/commit/2eb3442879d2fbad184a5f9a788c92555467e1e2))
- **externalizer**: Add TomlExternalizer and asb_dump binary([`a26f211`](https://github.com/frA-GRA/rcal/commit/a26f2118acbf1de571de42937f4941609c98b11d))
- **asb**: Add peers config for cross-process DISH connections([`cf3a549`](https://github.com/frA-GRA/rcal/commit/cf3a549085263afffb52f4c4f46aef18ed59cd75))
- Asb viewer added and some fixes to adsb_sim([`060f034`](https://github.com/frA-GRA/rcal/commit/060f0341bcf657bbfde0a691c4f99cb8032e4e87))
- **trace**: Port rcal_trace macro from 86-create-a-cal-bridge([`72616c4`](https://github.com/frA-GRA/rcal/commit/72616c4981ca8662671a47e186c903ec786857d1))
- **uci**: Add security_banner() and logfmt timestamp/level improvements([`a00f1af`](https://github.com/frA-GRA/rcal/commit/a00f1af698536fd8f69a49e373c178d8c3e05366))
### Refactoring

- **cal**: Remove AbstractCalExt, add Cal concrete struct with generic create_reader/create_writer([`ec2dbca`](https://github.com/frA-GRA/rcal/commit/ec2dbcafc02625bb625ae89a5be94c452257be88))


## [2.1.2] - 2026-09-02
### Bug Fixes

- Remove broken intra-doc link to CompressionExternalizer([`a53d764`](https://github.com/frA-GRA/rcal/commit/a53d7645ba59d0dcad8fa99ee7577249c55f59a8))


## [2.1.1] - 2026-09-02
### Bug Fixes

- Cleanup batch #62 #63 #64 #65 #74 #79([`ec4e8a6`](https://github.com/frA-GRA/rcal/commit/ec4e8a6ba93c5b51b5171ef7ef538589418bf044))
- Remove redundant ::default() call on unit struct XmlExternalizerLoader([`f2ab4a8`](https://github.com/frA-GRA/rcal/commit/f2ab4a8c414568f27981e0153d3722a0f08a5c81))


## [2.1.0] - 2026-08-24
### Bug Fixes

- Resolve P0 bugs in owner_producer mapping, service_status_loop macro, and DateTime overflow([`3aa7ac3`](https://github.com/frA-GRA/rcal/commit/3aa7ac36353a450d39c3b755eefc5ba26fa7f057))
- Resolve all P1 issues (safety, bug, docs)([`f639c6c`](https://github.com/frA-GRA/rcal/commit/f639c6c711f17592f87e899c45db8885b1802640))
- Lower test and example port numbers to 2000-range (review)([`7fa9bdb`](https://github.com/frA-GRA/rcal/commit/7fa9bdb4ed996a787018f698a1e3e840b8ea46ad))
- Implement AbstractService::reset() per CAL §5 lifecycle([`0384f01`](https://github.com/frA-GRA/rcal/commit/0384f01ad662a35a22197342de549175c2f5e19b))
### Documentation

- Add documentation to xsd-subset([`af4c24b`](https://github.com/frA-GRA/rcal/commit/af4c24b3f4214e3f2313a7a5583ddd662b87476b))
- Fix broken intra-doc links after CAL layer rename([`180d75e`](https://github.com/frA-GRA/rcal/commit/180d75e2a42944092cbe0c7338a621eb8a1427bc))
### Features

- **base**: Store and expose per-instance BoundedList bounds via const generics ([#52](https://github.com/frA-GRA/rcal/pull/52))([`6437b9c`](https://github.com/frA-GRA/rcal/commit/6437b9cc8009b687fa546f23eb3851d87c6ac18d))
- **asb**: Add UUID identity methods to AbstractServiceBus (issue #49)([`a3bf496`](https://github.com/frA-GRA/rcal/commit/a3bf496c3b644f1bfc893081de03767a68fe10da))
- Add xs::Base64Binary primitive type (closes #53)([`b96b31b`](https://github.com/frA-GRA/rcal/commit/b96b31bdf25cf48246192529a8e3a4be0731f5c7))
- Add AbstractServiceBus version and label methods (closes #51)([`24aba6b`](https://github.com/frA-GRA/rcal/commit/24aba6b42af877449a201ed78b65e3661c2f329e))
- Separate AbstractServiceBus from CAL layer (closes #72)([`978db85`](https://github.com/frA-GRA/rcal/commit/978db85216c23b6502167830f112335d1f619bf1))
- **service**: Add create_polling_reader returning Box<dyn AbstractReader> (closes #75)([`c0f1a34`](https://github.com/frA-GRA/rcal/commit/c0f1a34112288f404605c53bcc1bb1351b3ad423))
- **xs**: Add AnyUri, NormalizedString, Token type aliases (closes #77)([`46ca0db`](https://github.com/frA-GRA/rcal/commit/46ca0dbdf8c67bacc5a5d7f6c4ee089e6fee2ba9))
- **uci/base**: Add Externalizer/ExternalizerLoader abstraction (closes #54)([`23ff61f`](https://github.com/frA-GRA/rcal/commit/23ff61f6b0bc60435fb2a73a87407f026389c258))
- **externalizer**: Add ChainExternalizer, CompressionExternalizer, config-driven externalizer factory([`19ab467`](https://github.com/frA-GRA/rcal/commit/19ab46720d64dae34150980491a6cbfac2def6b5))
- **calconfig**: Add per-topic QoS config and apply reliability check (CAL-005210, CAL-005434)([`5476345`](https://github.com/frA-GRA/rcal/commit/54763455a5eaa2655ae0653ba6b6e06559ca0aa9))
- **codegen**: Add _set() and _disable() for optional fields (CAL-005290)([`66902d4`](https://github.com/frA-GRA/rcal/commit/66902d45d65cb2b33a2cfb9ee19962fa3eeb5921))
### Refactoring

- **asb**: Rename get_abstract_service_bus_connection_version to get_asb_connection_version([`a9b9887`](https://github.com/frA-GRA/rcal/commit/a9b9887ca235b5d7008e96c84360e551949b9d98))
- **externalizer**: Remove destroy_externalizer_loader; document drop([`25ad9dc`](https://github.com/frA-GRA/rcal/commit/25ad9dcc23153f4e1764076d75cfcaf83039d9b6))
- **externalizer**: Non-generic Externalizer trait, fluent builder, chain via next field([`a6aa419`](https://github.com/frA-GRA/rcal/commit/a6aa41902572955d53dc808f6faf26360182cc1e))

