import test from "ava";
import { modelFile, openJtalkDicDir, checkAllMoras } from "../_testUtil";
import { OpenJtalk, Synthesizer, VoiceModel } from "voicevox_core/blocking";

function loadOpenJtalk() {
  return OpenJtalk.create(openJtalkDicDir);
}

function loadModel() {
  return VoiceModel.fromPath(modelFile);
}

test("ハードウエアアクセラレーションモードが指定したとおりに設定されること", (t) => {
  const openJtalk = loadOpenJtalk();
  const synthesizer = new Synthesizer(openJtalk, {
    accelerationMode: "CPU",
  });
  t.false(synthesizer.isGpuMode);
});

test("VoiceModelのロード・アンロード時にモデル数を正しく取得できること", (t) => {
  const model = loadModel();
  t.true(model.metas.length >= 1);

  const openJtalk = loadOpenJtalk();
  const synthesizer = new Synthesizer(openJtalk);

  t.is(synthesizer.metas.length, 0);

  synthesizer.loadVoiceModel(model);
  t.is(synthesizer.metas.length, model.metas.length);

  synthesizer.unloadVoiceModel(model.id);

  t.is(synthesizer.metas.length, 0);
  t.false(synthesizer.isLoadedVoiceModel(model.id));
});

test("AudioQueryからの合成でエラーが発生しないこと", (t) => {
  const model = loadModel();
  const openJtalk = loadOpenJtalk();
  const synthesizer = new Synthesizer(openJtalk);
  synthesizer.loadVoiceModel(model);
  const query = synthesizer.audioQuery(
    "こんにちは",
    model.metas[0].styles[0].id,
  );
  t.notThrows(() => synthesizer.synthesis(query, model.metas[0].styles[0].id));
});

test("パラメータを変えてAccentPhraseを生成し直すとモーラの値が変わること", (t) => {
  const model = loadModel();
  const openJtalk = loadOpenJtalk();
  const synthesizer = new Synthesizer(openJtalk);
  synthesizer.loadVoiceModel(model);
  const accentPhrases = synthesizer.createAccentPhrases(
    "こんにちは",
    model.metas[0].styles[0].id,
  );
  const accentPhrases2 = synthesizer.replaceMoraPitch(
    accentPhrases,
    model.metas[1].styles[0].id,
  );
  t.true(
    checkAllMoras(
      accentPhrases,
      accentPhrases2,
      (mora, otherMora) => mora.pitch != otherMora.pitch,
    ),
  );
  const accentPhrases3 = synthesizer.replacePhonemeLength(
    accentPhrases,
    model.metas[1].styles[0].id,
  );
  t.true(
    checkAllMoras(
      accentPhrases,
      accentPhrases3,
      (mora, otherMora) => mora.vowel_length != otherMora.vowel_length,
    ),
  );
  const accentPhrases4 = synthesizer.replaceMoraData(
    accentPhrases,
    model.metas[1].styles[0].id,
  );
  t.true(
    checkAllMoras(
      accentPhrases,
      accentPhrases4,
      (mora, otherMora) =>
        mora.pitch != otherMora.pitch &&
        mora.vowel_length != otherMora.vowel_length,
    ),
  );
});

test("日本語のテキストからの音声合成でエラーが発生しないこと", (t) => {
  const model = loadModel();
  const openJtalk = loadOpenJtalk();
  const synthesizer = new Synthesizer(openJtalk);
  synthesizer.loadVoiceModel(model);
  t.notThrows(() => synthesizer.tts("こんにちは", model.metas[0].styles[0].id));
});
