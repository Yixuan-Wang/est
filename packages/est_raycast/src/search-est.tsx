import { useState } from "react";
import { Action, ActionPanel, Icon, List, getPreferenceValues } from "@raycast/api";
import { useFetch } from "@raycast/utils";

const PLACEHOLDER = {
  id: "placeholder",
  icon: Icon.Globe,
  title: "Open Est Web...",
};

interface Preferences {
  endpoint: string;
}

function parseSearchTemporary(searchText: string) {
  let [_, mention, content] = searchText.match(/^@((?:\w|\.)+)(?:\s+(.*))?$/) ?? [null, "", searchText];
  mention = mention ?? "";
  content = content ?? "";

  if (content === "@") {
    content = "";
  }

  const mentionSegments = mention.split(".");
  content = content.trim();

  return {
    mention: mentionSegments,
    mentionString: mention,
    content,
  };
}

function getSuggestions(content: string, execute: boolean = false) {
  try {
    const useGoogle = !content.startsWith("!") && !content.startsWith("\\");
    const search = useGoogle ? new URL("https://www.google.com/complete/search?output=firefox") : new URL("https://ac.duckduckgo.com/ac/");
    search.searchParams.set("q", content);
    if (!useGoogle) search.searchParams.set("type", "list");
    
    const { data } = useFetch(search.toString(), {
      execute,
      parseResponse: (response) => response.json(),
    });

    const suggestions: string[] = (data as any)?.[1] ?? [];
    return {
      using: useGoogle ? "Google" : "DuckDuckGo",
      suggestions
    }
  } catch {
    return {
      using: null,
      suggestions: [] as string[],
    }
  }
}

export default function Command() {
  const { endpoint } = getPreferenceValues<Preferences>();

  const [searchText, setSearchText] = useState("");

  const Placeholder = <List.Item {...PLACEHOLDER} accessories={[{ text: endpoint }]} actions={<ActionPanel>
    <Action.OpenInBrowser url={endpoint} title="Open" />
  </ActionPanel>} />;

  const { mention, content, mentionString } = parseSearchTemporary(searchText);
  const mentionAccessories = mention.map((m) => ({ tag: m }));

  const search = new URL("search", endpoint);
  search.searchParams.set("q", searchText);

  const DirectSearch = content ? (
    <List.Item
      id="search-direct"
      title={content}
      icon={Icon.MagnifyingGlass}
      accessories={mentionAccessories}
      actions={
        <ActionPanel>
          <Action.OpenInBrowser url={search.toString()} />
        </ActionPanel>
      }
    />
  ) : (
    <List.Item id="search-direct" title={`Using @${mention.join(".")}`} icon={Icon.Binoculars} />
  );

  const { suggestions, using } = getSuggestions(content, content !== "");

  return (
    <List searchBarPlaceholder="Search on Est" onSearchTextChange={setSearchText}>
      {searchText === "" ? Placeholder : DirectSearch}
      {suggestions.map((suggestion, idx) => {
        const url = new URL("search", endpoint);
        url.searchParams.set("q", `${mentionString ? `@${mentionString} ` : '' }${suggestion}`);
        return (<List.Item
          key={`suggestion-${idx}`}
          title={suggestion}
          icon={Icon.Stars}
          actions={
            <ActionPanel>
              <Action.OpenInBrowser title="Accept Suggestion" url={url.toString()} />
            </ActionPanel>
          }
        />);
      })}
    </List>
  );
}
