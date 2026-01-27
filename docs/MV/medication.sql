CREATE OR REPLACE VIEW "DBAMV"."MEDICATION"
(
    "medication_code",
    "medication_encounter_code",
    "medication_patient_code",
    "medication_provider_code",
    "medication_name",
    "medication_category",
    "medication_status",
    "medication_route",
    "medication_frequency",
    "medication_dosage",
    "medication_description",
    "medication_date",
    "medication_time",
    "medication_update_date",
    "medication_update_time",
    "medication_reason",
    "medication_responsible",
    "created_date",
    "created_time" 
) AS
SELECT 
    IM.CD_PRE_MED                          AS "medication_code",
    PM.CD_ATENDIMENTO                      AS "medication_encounter_code",
    P.CD_PACIENTE                          AS "medication_patient_code",
    PM.CD_PRESTADOR                        AS "medication_provider_code",
    TP.DS_TIP_PRESC                        AS "medication_name",
    'Prescrição'                           AS "medication_category",
    HC.SN_SUSPENSO                         AS "medication_status",
    FA.DS_FOR_APL                          AS "medication_route",
    TF.DS_TIP_FRE                          AS "medication_frequency",
    IM.QT_ITPRE_MED                        AS "medication_dosage",
    IM.DS_ITPRE_MED                        AS "medication_description",
    TO_CHAR(HC.DH_MEDICACAO, 'YYYY-MM-DD') AS "medication_date",
    TO_CHAR(HC.DH_MEDICACAO, 'HH24:MI:SS') AS "medication_time",
    NULL AS "medication_update_date",
    NULL AS "medication_update_time",
    NULL AS "medication_reason",
    NULL AS "medication_responsible",
    TO_CHAR(PM.DT_PRE_MED,'YYYY-MM-DD')   AS "created_date",
    TO_CHAR(PM.HR_PRE_MED,'HH24:MI:SS')   AS "created_time" 
FROM DBAMV.PRE_MED PM   
    INNER JOIN DBAMV.ITPRE_MED IM            ON IM.CD_PRE_MED = PM.CD_PRE_MED
    LEFT  JOIN dbamv.hritpre_cons hc         ON HC.CD_ITPRE_MED = IM.CD_ITPRE_MED
    INNER JOIN DBAMV.TIP_ESQ TE              ON IM.CD_TIP_ESQ = TE.CD_TIP_ESQ
    INNER JOIN DBAMV.TIP_PRESC TP            ON IM.CD_TIP_PRESC = TP.CD_TIP_PRESC
    INNER JOIN DBAMV.TIP_FRE TF              ON IM.CD_TIP_FRE = TF.CD_TIP_FRE
    INNER JOIN DBAMV.FOR_APL FA              ON IM.CD_FOR_APL = FA.CD_FOR_APL
    INNER JOIN DBAMV.ATENDIME A              ON A.CD_ATENDIMENTO = PM.CD_ATENDIMENTO
    INNER JOIN DBAMV.PACIENTE P              ON P.CD_PACIENTE = A.CD_PACIENTE
    INNER JOIN DBAMV.PW_DOCUMENTO_CLINICO PW ON PW.CD_DOCUMENTO_CLINICO = PM.CD_DOCUMENTO_CLINICO
WHERE TE.sn_tipo = 'S'
AND PW.TP_STATUS = 'ASSINADO';
                                     
                      
                                
                                     
                                     
                                     
                                     
                                     