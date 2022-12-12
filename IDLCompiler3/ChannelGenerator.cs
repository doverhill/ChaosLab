using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace IDLCompiler
{
    internal static class ChannelGenerator
    {
        public static void GenerateChannel(SourceGenerator source, IDL idl)
        {
            var protocolName = CasedString.FromSnake(idl.Protocol.Name);
            var channelName = $"{protocolName.ToPascal()}Channel";

            var versionBlock = source.AddBlock("struct ProtocolVersion");
            versionBlock.AddLine("major: u16,");
            versionBlock.AddLine("minor: u16,");
            versionBlock.AddLine("patch: u16,");
            versionBlock.AddLine("is_preview: bool,");
            versionBlock.AddLine("preview_version: u16,");

            source.AddBlank();

            var channelHeader = source.AddBlock("struct ChannelHeader");
            channelHeader.AddLine("lock: AtomicBool,");
            channelHeader.AddLine("channel_magic: u64,");
            channelHeader.AddLine("protocol_name: [u8; 32],");
            channelHeader.AddLine("protocol_version: ProtocolVersion,");
            channelHeader.AddLine("first_message_offset: usize,");
            channelHeader.AddLine("last_message_offset: usize,");
            channelHeader.AddLine("number_of_messages: usize,");
            channelHeader.AddLine("is_writing: bool,");

            source.AddBlank();

            var channeHeaderlImpl = source.AddBlock("impl ChannelHeader");
            channeHeaderlImpl.AddLine("pub const MAGIC: u64 = u64::from_be_bytes(['C' as u8, 'C' as u8, 'H' as u8, 'A' as u8, 'N' as u8, 'N' as u8, 'E' as u8, 'L' as u8]);");

            source.AddBlank();

            var messageHeader = source.AddBlock("pub struct ChannelMessageHeader");
            messageHeader.AddLine("message_magic: u64,");
            messageHeader.AddLine("message_id: u64,");
            messageHeader.AddLine("message_length: usize,");
            messageHeader.AddLine("previous_message_offset: usize,");
            messageHeader.AddLine("next_message_offset: usize,");
            messageHeader.AddLine("replace_pending: bool,");

            source.AddBlank();

            var messageImpl = source.AddBlock("impl ChannelHeader");
            messageImpl.AddLine("pub const MAGIC: u64 = u64::from_be_bytes(['C' as u8, 'M' as u8, 'E' as u8, 'S' as u8, 'S' as u8, 'A' as u8, 'G' as u8, 'E' as u8]);");

            source.AddBlank();

            var lockBlock = source.AddBlock("struct ChannelLock");
            lockBlock.AddLine("channel_header: *const ChannelHeader,");

            source.AddBlank();

            var lockImpl = source.AddBlock("impl ChannelLock");
            var getFunction = lockImpl.AddBlock($"pub fn get(channel: &{channelName}) -> Self");
            getFunction.AddLine("let channel_header = channel.channel_address as *const ChannelHeader;");
            getFunction.AddLine("while (*channel_header).lock.swap(true, Ordering::Acquire) {}");
            var getReturn = getFunction.AddBlock("Self");
            getReturn.AddLine("channel_header: channel_header");

            source.AddBlank();

            var dropImpl = source.AddBlock("impl Drop for ChannelLock");
            var dropFunction = dropImpl.AddBlock("fn drop(&mut self)");
            var dropUnsafe = dropFunction.AddBlock("unsafe");
            dropUnsafe.AddLine("(*self.channel_header).lock.swap(false, Ordering::Release);");

            source.AddBlank();

            var channel = source.AddBlock($"pub struct {channelName}");
            channel.AddLine("channel_address: *mut u8,");

            source.AddBlank();

            var channelImpl = source.AddBlock($"impl {channelName}");

            var newFunction = channelImpl.AddBlock("pub unsafe fn new(channel_address: *mut u8, is_server: bool) -> Self");

        }
    }
}
