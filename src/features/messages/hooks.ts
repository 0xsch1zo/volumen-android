import { useInfiniteQuery, useMutation, useQuery } from "@tanstack/react-query";
import {
    MESSAGE_PAGE_SIZE,
    messageDetail,
    messageList,
    sendMessage,
} from "./api";
import {
    MessageBox,
    ReceivedMessagesPage,
    SentMessagesPage,
} from "./types";

type MessagesPage = ReceivedMessagesPage | SentMessagesPage;

function useMessageList(box: MessageBox) {
    const query = useInfiniteQuery<
        MessagesPage,
        Error,
        MessagesPage,
        [string, MessageBox]
    >({
        queryKey: ["messages", box],
        queryFn: async ({ pageParam = 1 }) => messageList(box)(Number(pageParam), MESSAGE_PAGE_SIZE),
        getNextPageParam: (lastPage, allPages) => {
            const loaded = allPages.reduce((count, page) => count + page.messages.length, 0);
            return loaded < lastPage.total ? allPages.length + 1 : undefined;
        },
    });

    return query;
}

function useReceivedMessages() {
    return useMessageList("received");
}

function useSentMessages() {
    return useMessageList("sent");
}

function useMessages(box: MessageBox) {
    return useMessageList(box);
}

function useMessage(box: MessageBox, messageId: number) {
    return useQuery({
        queryKey: ["message", box, messageId],
        queryFn: () => messageDetail(box)(messageId),
        enabled: Number.isInteger(messageId) && messageId > 0,
    });
}

function useSendMessage() {
    return useMutation({
        mutationFn: ({ messageId, message }: { messageId: number; message: string }) =>
            sendMessage(messageId, message),
    });
}

export {
    useReceivedMessages,
    useSentMessages,
    useMessages,
    useMessage,
    useSendMessage,
};
