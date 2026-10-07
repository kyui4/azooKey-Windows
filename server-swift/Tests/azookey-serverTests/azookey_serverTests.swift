import Testing
import Foundation
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
