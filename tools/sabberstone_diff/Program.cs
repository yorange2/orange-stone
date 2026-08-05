using System;
using System.Collections.Generic;
using SabberStoneCore.Actions;
using SabberStoneCore.Config;
using SabberStoneCore.Enums;
using SabberStoneCore.Model;
using SabberStoneCore.Model.Entities;
using SabberStoneCore.Tasks.PlayerTasks;

class Program
{
    // 场景：4/5 攻击 2/3 —— 与 orange-stone tests/differential.rs
    // scenario_attack_trade_event_sequence 对照（路线图 F5 外部 SabberStone 对照）。
    static void Main()
    {
        var config = new GameConfig
        {
            StartPlayer = 1,
            Player1HeroClass = CardClass.MAGE,
            Player2HeroClass = CardClass.MAGE,
            FillDecks = false,
            Player1Deck = new List<Card>(),
            Player2Deck = new List<Card>(),
            RandomSeed = 12345,
        };
        var game = new Game(config);
        game.StartGame();

        // P1 回合：出 Chillwind Yeti (4/5)，结束
        var yetiCard = Cards.FromName("Chillwind Yeti");
        Generic.DrawCard(game.CurrentPlayer, yetiCard);
        var yeti = (Minion)game.CurrentPlayer.HandZone[game.CurrentPlayer.HandZone.Count - 1];
        yeti.Cost = 0;
        Console.WriteLine($"play yeti: {game.Process(PlayCardTask.Any(game.CurrentPlayer, yeti))}");
        game.Process(EndTurnTask.Any(game.CurrentPlayer));

        // P2 回合：出 River Crocolisk (2/3)，结束
        var crocCard = Cards.FromName("River Crocolisk");
        Generic.DrawCard(game.CurrentPlayer, crocCard);
        var croc = (Minion)game.CurrentPlayer.HandZone[game.CurrentPlayer.HandZone.Count - 1];
        croc.Cost = 0;
        Console.WriteLine($"play croc: {game.Process(PlayCardTask.Any(game.CurrentPlayer, croc))}");
        game.Process(EndTurnTask.Any(game.CurrentPlayer));

        // P1 回合：4/5 攻击 2/3
        Console.WriteLine($"current player: {game.CurrentPlayer.PlayerId}  turn: {game.Turn}");
        Console.WriteLine($"attacker zone {yeti.Zone} exhausted {yeti.IsExhausted}  canAttack {yeti.CanAttack}");
        Console.WriteLine($"defender zone {croc.Zone}");
        Console.WriteLine($"before: attacker {yeti.AttackDamage}/{yeti.Health}  defender {croc.AttackDamage}/{croc.Health}");
        var ok = game.Process(MinionAttackTask.Any(game.CurrentPlayer, yeti, croc));
        Console.WriteLine($"attack processed: {ok}");
        Console.WriteLine($"after: attacker health = {yeti.Health}");
        Console.WriteLine($"defender zone = {croc.Zone}");
        Console.WriteLine($"defender dead = {croc.IsDead}");
    }
}
