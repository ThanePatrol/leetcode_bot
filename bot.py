import discord

intents = discord.Intents.default()
intents.message_content = True

client = discord.Client(intents=intents)

@client.event
async def on_ready():
    print(f'logged in as {client.user}')

@client.event
async def on_message(message):
    if message.author == client.user:
        return
    await message.channel.send('Hello!')

client.run('MTExNDY4MzA2MjQxNzExNzIxNA.GR-cBT.8NFWs_jcOL2WgyZIgbaEKxI4HRrxGhF8cHkG_c')
