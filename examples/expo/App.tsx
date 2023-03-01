import { StatusBar } from "expo-status-bar";
import { SafeAreaView, ScrollView, Text, TextInput, View } from "react-native";
import documents from "./lib/graphql-client";
import useSWR from "swr";
import { styled } from "nativewind";
import { useState } from "react";
import * as Constants from "expo-constants";

const StyledTextInput = styled(TextInput);

export default function App() {
  const { data, isLoading } = useSWR("emojis", () => documents.Emojis());
  const [filter, setFilter] = useState("");

  return (
    <SafeAreaView className="flex-1 bg-black">
      <View className="mt-4">
        <View className="bg-black pb-8 px-4">
          <Text className="text-white text-3xl font-black">
            Grafbase Emojis <Text className="text-gray-500 text-sm">{Constants?.default?.expoConfig?.version}</Text>
          </Text>
          <StyledTextInput
            placeholder="🔎 Search"
            onChangeText={setFilter}
            placeholderTextColor="gray"
            className="border py-2 border-gray-800 mt-4 rounded-lg text-white font-semibold px-2 text-xl focus:border-green-500"
          />
        </View>
        <ScrollView className="bg-gray-900 min-h-screen gap-4 px-4 pb-4">
          {isLoading && (
            <Text className="text-gray-500">Loading emojis...</Text>
          )}
          {data?.data?.emojiCollection?.edges?.map((e) => {
            if (!e?.node) {
              return null;
            }

            const { char, tags } = e.node;

            if (
              !tags.edges?.some((e) =>
                filter ? e.node.text.includes(filter.toLowerCase()) : true
              )
            ) {
              return null;
            }

            return (
              <View
                key={char}
                className="h-32 flex items-center justify-center bg-black border border-gray-800 rounded-lg"
              >
                <View>
                  <Text className="text-5xl">{char}</Text>
                </View>
              </View>
            );
          })}
        </ScrollView>
      </View>

      <StatusBar style="light" />
    </SafeAreaView>
  );
}
