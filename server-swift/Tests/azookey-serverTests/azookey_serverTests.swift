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

