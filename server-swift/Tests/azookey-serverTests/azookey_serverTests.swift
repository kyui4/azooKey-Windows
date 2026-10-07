import Testing
import Foundation
import KanaKanjiConverterModule
@testable import azookey_server

@Test func dictionaryUsesKatakanaIndex() throws {
    let entry = UserDictionaryEntry(reading: "あずーきー", word: "azooKey")
    #expect(entry.dicdata.ruby == "アズーキー")
    #expect(entry.dicdata.word == "azooKey")
    let decoded = try JSONDecoder().decode([UserDictionaryEntry].self,
        from: Data("[{\"reading\":\"あずーきー\",\"word\":\"azooKey\"}]".utf8))
    #expect(decoded == [entry])
    #expect(UserDictionaryEntry(reading: "AIあしすたんと", word: "AIアシスタント").dicdata.ruby == "AIアシスタント")
    #expect(UserDictionaryEntry(reading: "azooきー", word: "azooKey").dicdata.ruby == "azooキー")
}

@Test @MainActor func mixedAlphabetAndKanaWordAppearsAndCanBeRemoved() {
    let engine = KanaKanjiConverter()
    var options = getOptions()
    options.dictionaryResourceURL = URL(filePath: FileManager.default.currentDirectoryPath)
        .appendingPathComponent("azooKey_dictionary_storage/Dictionary")
    options.requireJapanesePrediction = false
    options.zenzaiMode = .off
    for reading in ["AIあしすたんと", "azooきー", "あずーKey"] {
        let entry = UserDictionaryEntry(reading: reading, word: "混在読み辞書検証")
        var text = ComposingText()
        text.insertAtCursorPosition(reading, inputStyle: .direct)
        engine.sendToDicdataStore(.importDynamicUserDict([entry.dicdata]))
        let registered = engine.requestCandidates(text, options: options)
        #expect(registered.mainResults.contains { $0.text == entry.word })
        engine.stopComposition()
        engine.sendToDicdataStore(.importDynamicUserDict([]))
        let removed = engine.requestCandidates(text, options: options)
        #expect(!removed.mainResults.contains { $0.text == entry.word })
        engine.stopComposition()
    }
}

@Test func ffiResultsCanBeReleased() {
    let owned = "azooKey".withCString { strdup($0)! }
    #expect(String(cString: owned) == "azooKey")
    free_string(owned)
    free_candidates(to_list_pointer([]), 0)
}

@Test @MainActor func registeredWordAppearsAndCanBeRemoved() {
    let engine = KanaKanjiConverter()
    let entry = UserDictionaryEntry(reading: "こでっくす", word: "Codex辞書検証")
    let resources = URL(filePath: FileManager.default.currentDirectoryPath)
    var options = getOptions()
    options.dictionaryResourceURL = resources.appendingPathComponent("azooKey_dictionary_storage/Dictionary")
    options.requireJapanesePrediction = false
    options.zenzaiMode = .off
    var text = ComposingText()
    text.insertAtCursorPosition("kodekkusu", inputStyle: .roman2kana)
    engine.sendToDicdataStore(.importDynamicUserDict([entry.dicdata]))
    let registered = engine.requestCandidates(text, options: options)
    #expect(registered.mainResults.contains { $0.text == entry.word })
    engine.stopComposition()
    engine.sendToDicdataStore(.importDynamicUserDict([]))
    let removed = engine.requestCandidates(text, options: options)
    #expect(!removed.mainResults.contains { $0.text == entry.word })
}
