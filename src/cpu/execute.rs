use crate::{
    cpu::{Cpu, ExecResult},
    instruction::{DecodedInstruction, Instruction},
    trap::Trap,
};

pub mod csr_execute;
pub mod fp;
pub mod helper;
pub mod rv64a;
pub mod rv64d;
pub mod rv64f;
pub mod rv64i;
pub mod rv64m;
pub mod rv64u;
pub mod system;

pub fn execute(decoded: DecodedInstruction, cpu: &mut Cpu) -> ExecResult {
    match decoded.instruction {
        // ALU Type I
        Instruction::Addi { rd, rs1, imm } => rv64i::addi(cpu, rd, rs1, imm),
        Instruction::Slti { rd, rs1, imm } => rv64i::slti(cpu, rd, rs1, imm),
        Instruction::Sltiu { rd, rs1, imm } => rv64i::sltiu(cpu, rd, rs1, imm),
        Instruction::Xori { rd, rs1, imm } => rv64i::xori(cpu, rd, rs1, imm),
        Instruction::Ori { rd, rs1, imm } => rv64i::ori(cpu, rd, rs1, imm),
        Instruction::Andi { rd, rs1, imm } => rv64i::andi(cpu, rd, rs1, imm),
        Instruction::Slli { rd, rs1, shamt } => rv64i::slli(cpu, rd, rs1, shamt),
        Instruction::Srli { rd, rs1, shamt } => rv64i::srli(cpu, rd, rs1, shamt),
        Instruction::Srai { rd, rs1, shamt } => rv64i::srai(cpu, rd, rs1, shamt),
        // RV64-only immediate instructions (OP IMM 32)
        Instruction::Addiw { rd, rs1, imm } => rv64i::addiw(cpu, rd, rs1, imm),
        Instruction::Slliw { rd, rs1, shamt } => rv64i::slliw(cpu, rd, rs1, shamt),
        Instruction::Srliw { rd, rs1, shamt } => rv64i::srliw(cpu, rd, rs1, shamt),
        Instruction::Sraiw { rd, rs1, shamt } => rv64i::sraiw(cpu, rd, rs1, shamt),
        // register to register alu
        Instruction::Add { rd, rs1, rs2 } => rv64i::add(cpu, rd, rs1, rs2),
        Instruction::Sub { rd, rs1, rs2 } => rv64i::sub(cpu, rd, rs1, rs2),
        Instruction::Sll { rd, rs1, rs2 } => rv64i::sll(cpu, rd, rs1, rs2),
        Instruction::Srl { rd, rs1, rs2 } => rv64i::srl(cpu, rd, rs1, rs2),
        Instruction::Sra { rd, rs1, rs2 } => rv64i::sra(cpu, rd, rs1, rs2),
        Instruction::Slt { rd, rs1, rs2 } => rv64i::slt(cpu, rd, rs1, rs2),
        Instruction::Sltu { rd, rs1, rs2 } => rv64i::sltu(cpu, rd, rs1, rs2),
        Instruction::Xor { rd, rs1, rs2 } => rv64i::xor(cpu, rd, rs1, rs2),
        Instruction::Or { rd, rs1, rs2 } => rv64i::or(cpu, rd, rs1, rs2),
        Instruction::And { rd, rs1, rs2 } => rv64i::and(cpu, rd, rs1, rs2),
        // OP 32
        Instruction::Addw { rd, rs1, rs2 } => rv64i::addw(cpu, rd, rs1, rs2),
        Instruction::Subw { rd, rs1, rs2 } => rv64i::subw(cpu, rd, rs1, rs2),
        Instruction::Sllw { rd, rs1, rs2 } => rv64i::sllw(cpu, rd, rs1, rs2),
        Instruction::Srlw { rd, rs1, rs2 } => rv64i::srlw(cpu, rd, rs1, rs2),
        Instruction::Sraw { rd, rs1, rs2 } => rv64i::sraw(cpu, rd, rs1, rs2),
        // Jumps
        Instruction::Jal { rd, imm } => rv64i::jal(cpu, rd, imm),
        Instruction::Jalr { rd, rs1, imm } => rv64i::jalr(cpu, rd, rs1, imm),
        // Csr
        Instruction::Csrrw { rd, rs1, csr } => csr_execute::csrrw(cpu, rd, rs1, csr),
        Instruction::Csrrs { rd, rs1, csr } => csr_execute::csrrs(cpu, rd, rs1, csr),
        Instruction::Csrrc { rd, rs1, csr } => csr_execute::csrrc(cpu, rd, rs1, csr),
        Instruction::Csrrwi { rd, uimm, csr } => csr_execute::csrrwi(cpu, rd, uimm, csr),
        Instruction::Csrrsi { rd, uimm, csr } => csr_execute::csrrsi(cpu, rd, uimm, csr),
        Instruction::Csrrci { rd, uimm, csr } => csr_execute::csrrci(cpu, rd, uimm, csr),
        // Branches
        Instruction::Bne { rs1, rs2, imm } => rv64i::bne(cpu, rs1, rs2, imm),
        Instruction::Beq { rs1, rs2, imm } => rv64i::beq(cpu, rs1, rs2, imm),
        Instruction::Blt { rs1, rs2, imm } => rv64i::blt(cpu, rs1, rs2, imm),
        Instruction::Bge { rs1, rs2, imm } => rv64i::bge(cpu, rs1, rs2, imm),
        Instruction::Bltu { rs1, rs2, imm } => rv64i::bltu(cpu, rs1, rs2, imm),
        Instruction::Bgeu { rs1, rs2, imm } => rv64i::bgeu(cpu, rs1, rs2, imm),

        //Loads Type S
        Instruction::Lb { rd, rs1, imm } => rv64i::lb(cpu, rd, rs1, imm),
        Instruction::Lbu { rd, rs1, imm } => rv64i::lbu(cpu, rd, rs1, imm),
        Instruction::Lh { rd, rs1, imm } => rv64i::lh(cpu, rd, rs1, imm),
        Instruction::Lhu { rd, rs1, imm } => rv64i::lhu(cpu, rd, rs1, imm),
        Instruction::Lw { rd, rs1, imm } => rv64i::lw(cpu, rd, rs1, imm),
        Instruction::Lwu { rd, rs1, imm } => rv64i::lwu(cpu, rd, rs1, imm),
        Instruction::Ld { rd, rs1, imm } => rv64i::ld(cpu, rd, rs1, imm),

        //Store
        Instruction::Sb { rs1, rs2, imm } => rv64u::sb(cpu, rs1, rs2, imm),
        Instruction::Sh { rs1, rs2, imm } => rv64u::sh(cpu, rs1, rs2, imm),
        Instruction::Sw { rs1, rs2, imm } => rv64u::sw(cpu, rs1, rs2, imm),
        Instruction::Sd { rs1, rs2, imm } => rv64u::sd(cpu, rs1, rs2, imm),

        Instruction::Lui { rd, imm } => rv64u::lui(cpu, rd, imm),
        Instruction::Auipc { rd, imm } => rv64u::auipc(cpu, rd, imm),

        // System
        Instruction::Fence { pred, succ, fm } => system::fence(cpu, pred, succ, fm),
        Instruction::FenceI => system::fence_i(cpu),
        Instruction::Ebreak => system::ebreak(cpu),
        Instruction::Ecall => system::ecall(cpu),
        Instruction::Mret => system::mret(cpu),
        Instruction::Sret => system::sret(cpu),
        Instruction::SfenceVma { rs1, rs2 } => system::sfence_vma(cpu, rs1, rs2),
        Instruction::Wfi => system::wfi(cpu),

        // M extension
        Instruction::Mul { rd, rs1, rs2 } => rv64m::mul(cpu, rd, rs1, rs2),
        Instruction::Mulhu { rd, rs1, rs2 } => rv64m::mulhu(cpu, rd, rs1, rs2),
        Instruction::Mulh { rd, rs1, rs2 } => rv64m::mulh(cpu, rd, rs1, rs2),
        Instruction::Mulhsu { rd, rs1, rs2 } => rv64m::mulhsu(cpu, rd, rs1, rs2),
        Instruction::Divu { rd, rs1, rs2 } => rv64m::divu(cpu, rd, rs1, rs2),
        Instruction::Div { rd, rs1, rs2 } => rv64m::div(cpu, rd, rs1, rs2),
        Instruction::Rem { rd, rs1, rs2 } => rv64m::rem(cpu, rd, rs1, rs2),
        Instruction::Remu { rd, rs1, rs2 } => rv64m::remu(cpu, rd, rs1, rs2),
        Instruction::Mulw { rd, rs1, rs2 } => rv64m::mulw(cpu, rd, rs1, rs2),
        Instruction::Divw { rd, rs1, rs2 } => rv64m::divw(cpu, rd, rs1, rs2),
        Instruction::Divuw { rd, rs1, rs2 } => rv64m::divuw(cpu, rd, rs1, rs2),
        Instruction::Remw { rd, rs1, rs2 } => rv64m::remw(cpu, rd, rs1, rs2),
        Instruction::Remuw { rd, rs1, rs2 } => rv64m::remuw(cpu, rd, rs1, rs2),

        // A extension
        Instruction::Lrw { rd, rs1, rl: _rl, aq: _aq } => rv64a::lr_w(cpu, rd, rs1),
        Instruction::Lrd { rd, rs1, rl: _rl, aq: _aq } => rv64a::lr_d(cpu, rd, rs1),
        Instruction::Scw { rd, rs1, rs2, rl: _rl, aq: _aq } => rv64a::sc_w(cpu, rd, rs1, rs2),
        Instruction::Scd { rd, rs1, rs2, rl: _rl, aq: _aq } => rv64a::sc_d(cpu, rd, rs1, rs2),
        Instruction::Amoswapw { rd, rs1, rs2, rl: _rl, aq: _aq } => rv64a::amoswap_w(cpu, rd, rs1, rs2),
        Instruction::Amoswapd { rd, rs1, rs2, rl: _rl, aq: _aq } => rv64a::amoswap_d(cpu, rd, rs1, rs2),
        Instruction::Amoaddw { rd, rs1, rs2, rl: _rl, aq: _aq } => rv64a::amoadd_w(cpu, rd, rs1, rs2),
        Instruction::Amoaddd { rd, rs1, rs2, rl: _rl, aq: _aq } => rv64a::amoadd_d(cpu, rd, rs1, rs2),
        Instruction::Amoxorw { rd, rs1, rs2, rl: _rl, aq: _aq } => rv64a::amoxor_w(cpu, rd, rs1, rs2),
        Instruction::Amoxord { rd, rs1, rs2, rl: _rl, aq: _aq } => rv64a::amoxor_d(cpu, rd, rs1, rs2),
        Instruction::Amoorw { rd, rs1, rs2, rl: _rl, aq: _aq } => rv64a::amoor_w(cpu, rd, rs1, rs2),
        Instruction::Amoord { rd, rs1, rs2, rl: _rl, aq: _aq } => rv64a::amoor_d(cpu, rd, rs1, rs2),
        Instruction::Amoandw { rd, rs1, rs2, rl: _rl, aq: _aq } => rv64a::amoand_w(cpu, rd, rs1, rs2),
        Instruction::Amoandd { rd, rs1, rs2, rl: _rl, aq: _aq } => rv64a::amoand_d(cpu, rd, rs1, rs2),
        Instruction::Amominw { rd, rs1, rs2, rl: _rl, aq: _aq } => rv64a::amomin_w(cpu, rd, rs1, rs2),
        Instruction::Amomind { rd, rs1, rs2, rl: _rl, aq: _aq } => rv64a::amomin_d(cpu, rd, rs1, rs2),
        Instruction::Amomaxw { rd, rs1, rs2, rl: _rl, aq: _aq } => rv64a::amomax_w(cpu, rd, rs1, rs2),
        Instruction::Amomaxd { rd, rs1, rs2, rl: _rl, aq: _aq } => rv64a::amomax_d(cpu, rd, rs1, rs2),
        Instruction::Amominuw { rd, rs1, rs2, rl: _rl, aq: _aq } => rv64a::amominu_w(cpu, rd, rs1, rs2),
        Instruction::Amominud { rd, rs1, rs2, rl: _rl, aq: _aq } => rv64a::amominu_d(cpu, rd, rs1, rs2),
        Instruction::Amomaxuw { rd, rs1, rs2, rl: _rl, aq: _aq } => rv64a::amomaxu_w(cpu, rd, rs1, rs2),
        Instruction::Amomaxud { rd, rs1, rs2, rl: _rl, aq: _aq } => rv64a::amomaxu_d(cpu, rd, rs1, rs2),

        // F extension
        Instruction::Flw { rd, rs1, imm } => rv64f::flw(cpu, rd, rs1, imm),
        Instruction::Fsw { rs1, rs2, imm } => rv64f::fsw(cpu, rs1, rs2, imm),
        Instruction::FmvWX { rd, rs1 } => rv64f::fmv_w_x(cpu, rd, rs1),
        Instruction::FmvXW { rd, rs1 } => rv64f::fmv_x_w(cpu, rd, rs1),
        Instruction::FsgnjS { rd, rs1, rs2 } => rv64f::fsgnj_s(cpu, rd, rs1, rs2),
        Instruction::FsgnjnS { rd, rs1, rs2 } => rv64f::fsgnjn_s(cpu, rd, rs1, rs2),
        Instruction::FsgnjxS { rd, rs1, rs2 } => rv64f::fsgnjx_s(cpu, rd, rs1, rs2),
        Instruction::FclassS { rd, rs1 } => rv64f::fclass_s(cpu, rd, rs1),
        Instruction::FaddS { rd, rs1, rs2, rm } => rv64f::fadd_s(cpu, rd, rs1, rs2, rm),
        Instruction::FsubS { rd, rs1, rs2, rm } => rv64f::fsub_s(cpu, rd, rs1, rs2, rm),
        Instruction::FmulS { rd, rs1, rs2, rm } => rv64f::fmul_s(cpu, rd, rs1, rs2, rm),
        Instruction::FdivS { rd, rs1, rs2, rm } => rv64f::fdiv_s(cpu, rd, rs1, rs2, rm),
        Instruction::FsqrtS { rd, rs1, rm } => rv64f::fsqrt_s(cpu, rd, rs1, rm),
        Instruction::FminS { rd, rs1, rs2 } => rv64f::fmin_s(cpu, rd, rs1, rs2),
        Instruction::FmaxS { rd, rs1, rs2 } => rv64f::fmax_s(cpu, rd, rs1, rs2),
        Instruction::FeqS { rd, rs1, rs2 } => rv64f::feq_s(cpu, rd, rs1, rs2),
        Instruction::FltS { rd, rs1, rs2 } => rv64f::flt_s(cpu, rd, rs1, rs2),
        Instruction::FleS { rd, rs1, rs2 } => rv64f::fle_s(cpu, rd, rs1, rs2),
        Instruction::FcvtWS { rd, rs1, rm } => rv64f::fcvt_w_s(cpu, rd, rs1, rm),
        Instruction::FcvtWUS { rd, rs1, rm } => rv64f::fcvt_wu_s(cpu, rd, rs1, rm),
        Instruction::FcvtLS { rd, rs1, rm } => rv64f::fcvt_l_s(cpu, rd, rs1, rm),
        Instruction::FcvtLUS { rd, rs1, rm } => rv64f::fcvt_lu_s(cpu, rd, rs1, rm),
        Instruction::FcvtSW { rd, rs1, rm } => rv64f::fcvt_s_w(cpu, rd, rs1, rm),
        Instruction::FcvtSWU { rd, rs1, rm } => rv64f::fcvt_s_wu(cpu, rd, rs1, rm),
        Instruction::FcvtSL { rd, rs1, rm } => rv64f::fcvt_s_l(cpu, rd, rs1, rm),
        Instruction::FcvtSLU { rd, rs1, rm } => rv64f::fcvt_s_lu(cpu, rd, rs1, rm),
        Instruction::FmaddS { rd, rs1, rs2, rs3, rm } => rv64f::fmadd_s(cpu, rd, rs1, rs2, rs3, rm),
        Instruction::FmsubS { rd, rs1, rs2, rs3, rm } => rv64f::fmsub_s(cpu, rd, rs1, rs2, rs3, rm),
        Instruction::FnmsubS { rd, rs1, rs2, rs3, rm } => rv64f::fnmsub_s(cpu, rd, rs1, rs2, rs3, rm),
        Instruction::FnmaddS { rd, rs1, rs2, rs3, rm } => rv64f::fnmadd_s(cpu, rd, rs1, rs2, rs3, rm),

        // D extension
        Instruction::FcvtSD { rd, rs1, rm } => rv64d::fcvt_s_d(cpu, rd, rs1, rm),
        Instruction::Fld { rd, rs1, imm } => rv64d::fld(cpu, rd, rs1, imm),
        Instruction::Fsd { rs1, rs2, imm } => rv64d::fsd(cpu, rs1, rs2, imm),
        Instruction::FmvDX { rd, rs1 } => rv64d::fmv_d_x(cpu, rd, rs1),
        Instruction::FmvXD { rd, rs1 } => rv64d::fmv_x_d(cpu, rd, rs1),
        Instruction::FaddD { rd, rs1, rs2, rm } => rv64d::fadd_d(cpu, rd, rs1, rs2, rm),
        Instruction::FsubD { rd, rs1, rs2, rm } => rv64d::fsub_d(cpu, rd, rs1, rs2, rm),
        Instruction::FmulD { rd, rs1, rs2, rm } => rv64d::fmul_d(cpu, rd, rs1, rs2, rm),
        Instruction::FdivD { rd, rs1, rs2, rm } => rv64d::fdiv_d(cpu, rd, rs1, rs2, rm),
        Instruction::FsqrtD { rd, rs1, rm } => rv64d::fsqrt_d(cpu, rd, rs1, rm),
        Instruction::FsgnjD { rd, rs1, rs2 } => rv64d::fsgnj_d(cpu, rd, rs1, rs2),
        Instruction::FsgnjnD { rd, rs1, rs2 } => rv64d::fsgnjn_d(cpu, rd, rs1, rs2),
        Instruction::FsgnjxD { rd, rs1, rs2 } => rv64d::fsgnjx_d(cpu, rd, rs1, rs2),
        Instruction::FminD { rd, rs1, rs2 } => rv64d::fmin_d(cpu, rd, rs1, rs2),
        Instruction::FmaxD { rd, rs1, rs2 } => rv64d::fmax_d(cpu, rd, rs1, rs2),
        Instruction::FeqD { rd, rs1, rs2 } => rv64d::feq_d(cpu, rd, rs1, rs2),
        Instruction::FleD { rd, rs1, rs2 } => rv64d::fle_d(cpu, rd, rs1, rs2),
        Instruction::FltD { rd, rs1, rs2 } => rv64d::flt_d(cpu, rd, rs1, rs2),
        Instruction::FclassD { rd, rs1 } => rv64d::fclass_d(cpu, rd, rs1),
        Instruction::FcvtWD { rd, rs1, rm } => rv64d::fcvt_w_d(cpu, rd, rs1, rm),
        Instruction::FcvtWUD { rd, rs1, rm } => rv64d::fcvt_wu_d(cpu, rd, rs1, rm),
        Instruction::FcvtLD { rd, rs1, rm } => rv64d::fcvt_l_d(cpu, rd, rs1, rm),
        Instruction::FcvtLUD { rd, rs1, rm } => rv64d::fcvt_lu_d(cpu, rd, rs1, rm),
        Instruction::FcvtDW { rd, rs1, rm } => rv64d::fcvt_d_w(cpu, rd, rs1, rm),
        Instruction::FcvtDWU { rd, rs1, rm } => rv64d::fcvt_d_wu(cpu, rd, rs1, rm),
        Instruction::FcvtDL { rd, rs1, rm } => rv64d::fcvt_d_l(cpu, rd, rs1, rm),
        Instruction::FcvtDLU { rd, rs1, rm } => rv64d::fcvt_d_lu(cpu, rd, rs1, rm),
        Instruction::FcvtDS { rd, rs1, rm } => rv64d::fcvt_d_s(cpu, rd, rs1, rm),
        Instruction::FmaddD { rd, rs1, rs2, rs3, rm } => rv64d::fmadd_d(cpu, rd, rs1, rs2, rs3, rm),
        Instruction::FmsubD { rd, rs1, rs2, rs3, rm } => rv64d::fmsub_d(cpu, rd, rs1, rs2, rs3, rm),
        Instruction::FnmsubD { rd, rs1, rs2, rs3, rm } => rv64d::fnmsub_d(cpu, rd, rs1, rs2, rs3, rm),
        Instruction::FnmaddD { rd, rs1, rs2, rs3, rm } => rv64d::fnmadd_d(cpu, rd, rs1, rs2, rs3, rm),

        Instruction::Undefined { raw } => Err(Trap::IllegalInstruction(raw)),
        // _ => {
        //     panic!("Unimplemented instruction: {:#?}", decoded.instruction)
        // }
    }
}
