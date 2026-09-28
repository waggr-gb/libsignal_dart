// Every failure a caller recovers from arrives as a LibSignalException whose
// `code` names the condition, so nobody has to match on libsignal's English.
//
// Each test drives the real failure through the public API rather than
// constructing the exception, because the property that matters is that the
// code survives the whole path: libsignal -> this crate -> the bridge -> Dart.
// ignore_for_file: avoid_redundant_argument_values
import 'dart:convert';
import 'dart:typed_data';

import 'package:libsignal/libsignal.dart';
import 'package:test/test.dart';

import '../test_helpers/session_helpers.dart';
import '../test_helpers/test_party.dart';

Matcher _throwsCode(LibSignalErrorCode code) =>
    throwsA(isA<LibSignalException>().having((e) => e.code, 'code', code));

void main() {
  setUpAll(LibSignal.init);

  group('LibSignalException.code', () {
    late TestParty alice;
    late TestParty bob;
    late ProtocolAddress aliceAddress;

    setUp(() {
      alice = TestParty.create(name: 'alice', registrationId: 111);
      bob = TestParty.create(name: 'bob', registrationId: 222)
        ..generatePreKeys();
      aliceAddress = ProtocolAddress(name: 'alice', deviceId: 1);
    });

    Future<void> establish() async {
      await alice.sessionBuilder.processPreKeyBundle(
        bob.address,
        bob.getBundle(),
      );
      final first = await alice.sessionCipher.encrypt(
        bob.address,
        utf8.encode('first'),
      );
      await bob.sessionCipher.decrypt(aliceAddress, first);
      final reply = await bob.sessionCipher.encrypt(
        aliceAddress,
        utf8.encode('reply'),
      );
      await alice.sessionCipher.decrypt(bob.address, reply);
    }

    test('encrypting with no session is sessionNotFound', () async {
      await expectLater(
        alice.sessionCipher.encrypt(bob.address, utf8.encode('hi')),
        _throwsCode(LibSignalErrorCode.sessionNotFound),
      );
    });

    test('decrypting with no session is sessionNotFound', () async {
      await establish();
      final message = await alice.sessionCipher.encrypt(
        bob.address,
        utf8.encode('hi'),
      );
      final stranger = TestParty.create(name: 'carol', registrationId: 333);
      await expectLater(
        stranger.sessionCipher.decryptSignalMessage(
          aliceAddress,
          message.ciphertext,
        ),
        _throwsCode(LibSignalErrorCode.sessionNotFound),
      );
    });

    test('a replayed message is duplicatedMessage', () async {
      await establish();
      final message = await alice.sessionCipher.encrypt(
        bob.address,
        utf8.encode('once'),
      );
      await bob.sessionCipher.decryptSignalMessage(
        aliceAddress,
        message.ciphertext,
      );
      await expectLater(
        bob.sessionCipher.decryptSignalMessage(
          aliceAddress,
          message.ciphertext,
        ),
        _throwsCode(LibSignalErrorCode.duplicatedMessage),
      );
    });

    test('a tampered Whisper message is invalidWhisperMessage', () async {
      await establish();
      final message = await alice.sessionCipher.encrypt(
        bob.address,
        utf8.encode('intact'),
      );
      // Flip a bit in the trailing MAC: the envelope still parses, the MAC
      // does not verify against any session state.
      final tampered = Uint8List.fromList(message.ciphertext);
      tampered[tampered.length - 1] ^= 0x01;
      await expectLater(
        bob.sessionCipher.decryptSignalMessage(aliceAddress, tampered),
        _throwsCode(LibSignalErrorCode.invalidWhisperMessage),
      );
    });

    test('bytes that are not a Signal message are rejected by version or '
        'encoding, never as Other', () async {
      await establish();
      // Version nibble 1: older than anything libsignal still accepts.
      final legacy = Uint8List(64)..[0] = 0x11;
      await expectLater(
        bob.sessionCipher.decryptSignalMessage(aliceAddress, legacy),
        _throwsCode(LibSignalErrorCode.legacyCiphertextVersion),
      );
      // Version nibble 15: newer than this build understands.
      final future = Uint8List(64)..[0] = 0xFF;
      await expectLater(
        bob.sessionCipher.decryptSignalMessage(aliceAddress, future),
        _throwsCode(LibSignalErrorCode.unrecognizedCiphertextVersion),
      );
    });

    test('a missing signed pre-key is invalidSignedPreKeyId', () async {
      await alice.sessionBuilder.processPreKeyBundle(
        bob.address,
        bob.getBundle(),
      );
      final message = await alice.sessionCipher.encrypt(
        bob.address,
        utf8.encode('hi'),
      );
      final ids = extractPrekeyMessageIds(message: message.ciphertext);
      await bob.signedPreKeyStore.removeSignedPreKey(ids.signedPreKeyId);
      await expectLater(
        bob.sessionCipher.decryptPreKeyMessage(
          aliceAddress,
          message.ciphertext,
        ),
        _throwsCode(LibSignalErrorCode.invalidSignedPreKeyId),
      );
    });

    test('a changed remote identity is untrustedIdentity', () async {
      final bobAddress = ProtocolAddress(name: 'bob', deviceId: 1);
      final first = generateRemotePartyKeys(
        registrationId: 222,
        deviceId: 1,
        preKeyId: 1,
        signedPreKeyId: 1,
        kyberPreKeyId: 1,
      );
      final second = generateRemotePartyKeys(
        registrationId: 222,
        deviceId: 1,
        preKeyId: 2,
        signedPreKeyId: 2,
        kyberPreKeyId: 2,
      );
      await alice.sessionBuilder.processPreKeyBundle(
        bobAddress,
        first.toBundle(),
      );
      await expectLater(
        alice.sessionBuilder.processPreKeyBundle(bobAddress, second.toBundle()),
        _throwsCode(LibSignalErrorCode.untrustedIdentity),
      );
    });

    test('toString() is still the message it used to throw', () async {
      try {
        await alice.sessionCipher.encrypt(bob.address, utf8.encode('hi'));
        fail('expected a throw');
      } on LibSignalException catch (e) {
        expect(e.toString(), e.message);
        expect(e.message, startsWith('No session for bob:'));
      }
    });
  });
}
